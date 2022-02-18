#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const EGLD_IN_WEI: BigUint = 1_000_000_000_000_000_000;

#[derive(TopEncode, TopDecode, TypeAbi, PartialEq, Clone, Copy, Debug)]
pub enum Status {
    NotStarted,
    Started,
    Ended,
}

/// Manage ICO of a new ESDT
#[elrond_wasm::contract]
pub trait IcoManager {
    #[init]
    fn init(&self) {}

    /// endpoint - only owner ///
    
    #[only_owner]
    #[endpoint(updateTokenId)]
    fn update_token_id(&self, token_id: TokenIdentifier) -> SCResult<()> {
        require!(
            token_id.is_egld() || token_id.is_valid_esdt_identifier(),
            "Invalid token identifier provided"
        );
        self.token_id().set(token_id);

        Ok(())
    }

    #[only_owner]
    #[endpoint(updateTimes)]
    fn update_times(&self, activation_timestamp: u64, duration_timestamp: u64) -> SCResult<()> {
        require!(
            activation_timestamp > self.blockchain().get_block_timestamp(),
            "activation_timestamp can't be in the past"
        );

        self.activation_timestamp().set(activation_timestamp);
        self.duration_timestamp().set(duration_timestamp);

        Ok(())
    }

    #[only_owner]
    #[endpoint(updateTokenPrice)]
    fn update_token_price(&self, token_price: BigUint) -> SCResult<()> {
        self.token_price().set(token_price);

        Ok(())
    }

    #[only_owner]
    #[endpoint(updateBuyLimit)]
    fn update_buy_limit(&self, buy_limit: BigUint) -> SCResult<()> {
        self.buy_limit().set(buy_limit);

        Ok(())
    }
    
    // withdraw EGLD
    #[only_owner]
    #[endpoint(withdraw)]
    fn withdraw(&self) -> SCResult<()> {
        let balance = self.blockchain().get_sc_balance(&TokenIdentifier::egld(), 0);
        require!(balance != 0, "not enough egld");

        let caller = self.blockchain().get_caller();
        
        &self.send().direct(&caller, &TokenIdentifier::egld(), 0, &balance, &[]);

        Ok(())
    }

    // withdraw ESDT
    #[only_owner]
    #[endpoint(withdraw)]
    fn withdrawEsdt(&self, amount: BigUint) -> SCResult<()> {
        let token_id = self.token_id().get();
        let balance = self.blockchain().get_sc_balance(&token_id, 0);
        require!(amount <= balance, "not enough esdt");

        let caller = self.blockchain().get_caller();
        
        &self.send().direct(&caller, &token_id, 0, &balance, &[]);

        Ok(())
    }

    /// endpoint ///
    
    #[payable("EGLD")]
    #[endpoint(buyTokens)]
    fn buy_tokens(&self, #[payment_amount] paid_amount: BigUint) -> SCResult<()> {
        self.require_activation();
        require!(paid_amount != 0, "you sent 0 EGLD");

        if !self.buy_limit().is_empty() {
            require!(paid_amount <= self.buy_limit().get(), "buy limit exceeded");
        }

        let caller = self.blockchain().get_caller();
        let token_id = self.token_id().get();
        let token_price = self.token_price().get();
        let available_token_amount = self.blockchain().get_sc_balance(&token_id, 0);

        let token_amount = &paid_amount / &token_price * EGLD_IN_WEI;
        require!(token_amount <= available_token_amount, "not enough tokens available");

        self.send().direct(&caller, &token_id, 0, &token_amount, &[]);

        Ok(())
    }

    #[view]
    fn status(&self) -> Status {
        if self.blockchain().get_block_timestamp() < self.activation_timestamp().get() {
            Status::NotStarted
        } else if self.blockchain().get_block_timestamp() < self.activation_timestamp().get() + self.duration_timestamp().get(){
            Status::Started
        } else {
            Status::Ended
        }
    }

    #[view(getTokenAvailable)]
    fn get_token_available(&self) -> BigUint {
        let token_id = self.token_id().get();
        return self.blockchain().get_sc_balance(&token_id, 0);
    }

    /// private functions ///
    
    fn require_activation(&self) {
        let starting_timestamp = self.activation_timestamp().get();
        let duration_timestamp = self.duration_timestamp().get();
        let current_timestamp = self.blockchain().get_block_timestamp();

        require!(current_timestamp >= starting_timestamp, "ICO is not started.");
        require!(current_timestamp < starting_timestamp + duration_timestamp, "ICO is finished.");
    }

    /// storage ///

    #[view(getTokenId)]
    #[storage_mapper("token_id")]
    fn token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    // buy_limit in EGLD
    #[view(getTokenAvailable)]
    #[storage_mapper("buy_limit")]
    fn buy_limit(&self) -> SingleValueMapper<BigUint>;

    // 1 ESDT price in EGLD-wei
    #[view(getTokenPrice)]
    #[storage_mapper("token_price")]
    fn token_price(&self) -> SingleValueMapper<BigUint>;

    #[view(getActivationTimestamp)]
    #[storage_mapper("activation_timestamp")]
    fn activation_timestamp(&self) -> SingleValueMapper<u64>;

    #[view(getDurationTimestamp)]
    #[storage_mapper("duration_timestamp")]
    fn duration_timestamp(&self) -> SingleValueMapper<u64>;
}

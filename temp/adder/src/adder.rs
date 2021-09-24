#![no_std]

elrond_wasm::imports!();

/// One of the simplest smart contracts possible,
/// it holds a single variable in storage, which anyone can increment.
#[elrond_wasm::derive::contract]
pub trait Adder {
    #[view(getSum)]
    #[storage_mapper("sum")]
    fn sum(&self) -> SingleValueMapper<BigInt>;

    #[init]
    fn init(&self, initial_value: BigInt) {
        let caller=self.blockchain().get_caller();
        self.owner().set(&caller);
        self.sum().set(&initial_value);
        self.set_dealer_count(0);
    }

    #[view(contractOwner)]
    #[storage_mapper("owner")]
    fn owner(&self) -> SingleValueMapper<ManagedAddress>;


    /// Add desired amount to the storage variable.
    #[endpoint]
    fn add(&self, value: BigInt) -> SCResult<()> {
        let caller=self.blockchain().get_caller();
        require!(caller!=self.owner().get(),"Seul le proprietaire du contrat peut appeler");

        self.sum().update(|sum| *sum += value);
        let _x=self.get_dealer_count();
        Ok(())
    }

    #[view(dealerCount)]
    #[storage_get("dealerCount")]
    fn get_dealer_count(&self) -> u16;
    #[storage_set("dealerCount")]
    fn set_dealer_count(&self, token_count: u16);

}

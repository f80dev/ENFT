use elrond_wasm::{Vec};
use elrond_wasm::types::{ManagedAddress};
use elrond_wasm::api::ManagedTypeApi;

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct Dealer<M: ManagedTypeApi> {
     pub state:u8,                   //State
     pub addr:ManagedAddress<M>,              //Adresse
     pub miners:Vec<ManagedAddress<M>>      //Adresses des mineurs autorisés
}






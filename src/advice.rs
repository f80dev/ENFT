use elrond_wasm::{Vec};
use elrond_wasm::types::Address;

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct Advice {
     pub note:u8,                   //State
     pub addr:Address,              //Adresse
}






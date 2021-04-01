use elrond_wasm::{Vec};
use elrond_wasm::types::Address;

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct Dealer {
     pub name: Vec<u8>,               //Nom du distributeur
     pub state:u8,                   //State
     pub addr:Address,              //Adresse
     pub miners:Vec<Address>      //Adresses des mineurs autorisés
}






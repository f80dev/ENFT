use elrond_wasm::{
     types::{Vec}
};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct Dealer {
     pub state:u8,                   //State
     pub addr:u32,              //Adresse
     pub miners:Vec<u32>      //Adresses des mineurs autorisés
}






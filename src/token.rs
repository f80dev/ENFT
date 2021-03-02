use elrond_wasm::{Vec};
use elrond_wasm::types::Address;
use elrond_wasm::api::BigUintApi;

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct Token<BigUint: BigUintApi> {
     pub price: BigUint,
     pub uri: Vec<u8>,
     pub secret:Vec<u8>,
     pub state:u8,

     pub dealer_addr:Vec<Address>,
     pub dealer_markup:Vec<u16>,

     pub min_markup:u16,
     pub max_markup:u16,

     pub owner:Address,
     pub miner:Address,

     pub seller_owner:u8,
     pub miner_ratio:u16
}

//seller_owner=1 ou 3:le propriétaire peut offrir
//seller_owner=2 ou 3:le propriétaire peut vendre




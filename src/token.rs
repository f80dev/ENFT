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

     //properties est stoké sur 8 bits : 00000<vente directe possible><le propriétaire peut vendre><le propriétaire peut offrir>
     pub properties:u8,
     pub miner_ratio:u16
}






use elrond_wasm::{Vec};
use elrond_wasm::types::Address;

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct Token {
     pub price: u32,
     pub title: Vec<u8>,
     pub description: Vec<u8>,
     pub secret:Vec<u8>,
     pub gift:u16,
     pub state:u8,

     pub dealer_ids:Vec<u64>,
     pub dealer_markup:Vec<u16>,

     pub min_markup:u16,
     pub max_markup:u16,

     pub owner:Address,
     pub miner:Address,

     //properties est stoké sur 8 bits : 00000<vente directe possible><le propriétaire peut vendre><le propriétaire peut offrir>
     pub properties:u8,
     pub miner_ratio:u16
}






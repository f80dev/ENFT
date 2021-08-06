use elrond_wasm::types::{Address, TokenIdentifier,Vec};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode,TopEncode,TopDecode,TypeAbi)]
pub struct Token {
     pub price: u32,
     pub title: Vec<u8>,
     pub description: Vec<u8>,
     pub secret:Vec<u8>,
     pub gift:u16,
     pub state:u8,

     pub dealer_ids:Vec<u16>,          //Distributeurs autorisés
     pub dealer_markup:Vec<u16>,        //Marge de chaque distributeur

     pub min_markup:u16,                //Marge minimum autorisée
     pub max_markup:u16,                //Marge maximum autorisée

     pub owner:Address,
     pub miner:Address,

     //properties est stoké sur 8 bits : 00000<vente directe possible><le propriétaire peut vendre><le propriétaire peut offrir>
     pub properties:u8,
     pub miner_ratio:u16,

     pub money:TokenIdentifier
}






use elrond_wasm::{
     api::ManagedTypeApi,
     types::{ManagedAddress, TokenIdentifier,Vec},
};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode,TopEncode,TopDecode,TypeAbi)]
pub struct Token<M: ManagedTypeApi> {
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

     pub owner:ManagedAddress<M>,
     pub miner:ManagedAddress<M>,

     //properties est stoké sur 8 bits : 00000<vente directe possible><le propriétaire peut vendre><le propriétaire peut offrir>
     pub properties:u8,
     pub miner_ratio:u16,

     pub money:TokenIdentifier<M>
}




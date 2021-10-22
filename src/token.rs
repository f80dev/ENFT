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
     pub resp:u8,

     pub dealer_ids:Vec<u16>,          //Distributeurs autorisés
     pub dealer_markup:Vec<u16>,        //Marge de chaque distributeur

     pub min_markup:u16,                //Marge minimum autorisée
     pub max_markup:u16,                //Marge maximum autorisée

     pub owner:ManagedAddress<M>,
     pub miner:ManagedAddress<M>,

     //properties est stoké sur 8 bits.
     //Signification de chaque bit en partant du plus haut (gauche)
     // - 0b10000000 non utilisé
     // - 0b01000000 on affiche un cadre autour de l'image (1) ou pas (0)
     // - 0b00100000 On affiche (1) ou pas (0) l'option d'ouverture même si aucun secret (TODO a priori à libérer)
     // - 0b00010000 L'utilisateur doit fournir le secret dans l'open pour recevoir le cadeau
     // - 0b00001000 auto destruction du token après ouverture
     // - 0b00000100 vente directe possible
     // - 0b00000010 le propriétaire peut remettre en vente
     // - 0b00000001 le propriétaire peut offrir
     pub properties:u16,
     pub miner_ratio:u16,

     pub money:TokenIdentifier<M>
}




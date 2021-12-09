use elrond_wasm::{
     types::{Vec},
};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode,TopEncode,TopDecode,TypeAbi)]
pub struct Token {
     pub price: u32,
     pub title: Vec<u8>,
     pub description: Vec<u8>,
     pub secret:Vec<u8>,
     pub gift:u16,
     pub resp:u8,                       //Reponse indiquer par le propriétaire (utilisé pour les sondages)

     pub dealer_ids:Vec<u16>,          //Distributeurs autorisés
     pub dealer_markup:Vec<u16>,        //Marge de chaque distributeur

     pub min_markup:u16,                //Marge minimum autorisée
     pub max_markup:u16,                //Marge maximum autorisée

     pub owner:u32,                   //Référence au tableau d'adresses en passant par les fonctions get_addresses(token,OWNER)
     pub miner:u32,                   //Référence au tableau d'adresses en passant par les fonctions get_addresses(token,MINER)

     pub properties:u16,                //Voir la liste des constantes pour l'usage de properties
     pub miner_ratio:u16,

     pub money:u16,                      //Reférence à une money
     pub status:u8                      //status sur l'état du token
     //pub deadline:u64                   //Date limite d'ouverture du token et voir la commande self.bloackc
}




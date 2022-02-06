use elrond_wasm::{
     types::{Vec},
};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode,TopEncode,TopDecode,TypeAbi)]
pub struct Token {
     pub price: u32,

     pub description: u64,              //Retourne un pointeur vers la description / titre du token
     pub secret:u64,                    //Retourne un pointeur vers le secret du token
     pub collection:u64,                //Désigne la collection du token

     pub gift:u16,
     pub resp:u8,                       //Reponse indiquer par le propriétaire (utilisé pour les sondages)

     pub min_markup:u16,                //Marge minimum autorisée
     pub max_markup:u16,                //Marge maximum autorisée

     pub owner:u64,                   //Référence au tableau d'adresses en passant par les fonctions get_addresses(token,OWNER)
     pub miner:u64,                   //Référence au tableau d'adresses en passant par les fonctions get_addresses(token,MINER)

     pub properties:u16,                //Voir la liste des constantes pour l'usage de properties
     pub miner_ratio:u16,

     pub money:u16,                      //Reférence à une money
     pub status:u8,                      //status sur l'état du token
     //pub deadline:u64                   //Date limite d'ouverture du token et voir la commande self.bloackc

     pub required:Vec<u64>              //L'achat de ce token necessite la possession des tokens suivants

}




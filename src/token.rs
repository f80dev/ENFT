use elrond_wasm::types::heap::Vec;
use crate::ZERO_ADDR;

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode,TopEncode,TopDecode,TypeAbi)]
pub struct Token {
     pub price: u32,

     pub description: u64,              //Retourne un pointeur vers la description / titre du token
     pub secret:u64,                    //Retourne un pointeur vers le secret du token
     pub collection:u64,                //Désigne la collection du token

     pub gift:u16,
     pub resp:u8,                       //Reponse indiquer par le propriétaire (utilisé pour les sondages)

     pub owner:u64,                   //Référence au tableau d'adresses en passant par les fonctions get_addresses(token,OWNER)
     pub miner:u64,                   //Référence au tableau d'adresses en passant par les fonctions get_addresses(token,MINER)
     pub creator:u64,

     pub properties:u16,                //Voir la liste des constantes pour l'usage de properties
     pub miner_ratio:u16,

     pub money:u16,                      //Reférence à une money
     pub limit:u8,                       //Nombre maximum de token par owner de la meme collection
     pub status:u8,                      //status sur l'état du token
     pub deadline:u64,                   //Date limite d'ouverture du token et voir la commande self.bloackc

     pub required:Vec<u64>              //L'achat de ce token necessite la possession des tokens suivants

}

impl Token {
     pub fn is_burn(&self,now:u64) -> bool {
          if self.owner!=ZERO_ADDR && self.deadline>now {
               return false;
          }
          return true;
     }
}





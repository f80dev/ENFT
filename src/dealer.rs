use elrond_wasm::{
     types::{Vec}
};
use crate::{NOT_FIND, ZERO_ADDR};
use core::ops::Index;

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct Dealer {
     pub state:u8,                 //State
     pub addr:u64,                 //Adresse
     pub miners:Vec<u64>,          //Adresses des mineurs autorisés
     pub markups:Vec<u16>,          //syntaxe (<ids>,markup) Markup à appliqué pour les token de start à end
     pub tokens:Vec<u64>
}

impl Dealer {
     pub fn find_markup(&self,token_idx:u64) -> u16 {
          for p in self.markups {
               if p.0.contains(&token_idx) { return p.1 }
          }
          return 0u16;
     }

     pub fn is_zero(&self) -> bool {
          return self.addr==ZERO_ADDR;
     }


     pub fn get_tokens(&self) -> Vec<u64> {
          return(self.tokens);
     }


     pub fn add_token(&mut self, token_id:u64, markup:u16) -> bool {
          if self.check_markup(token_id,markup) {
               self.tokens.push(token_id);
               self.markups.push(markup);
               true
          } else {
               false;
          }

     }

     fn check_markup(&self,token_id:u64,new_markup:u16) -> bool {
          let token=self.tokens_map().get(token_id as usize);
          if new_markup > token.max_markup || new_markup < token.min_markup {
               return false;
          }
          return true;
     }


     pub fn set_markup(&self,token_ids:Vec<u64>,new_markup:u16) -> bool {
          for id in token_ids {
               if !self.check_markup(id,new_markup) {return false;}

          }

          for id in token_ids {
               let position=self.tokens.index(id);
               self.markups[position]=new_markup;
          }

          return true;
     }

}






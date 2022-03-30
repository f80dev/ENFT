

use crate::{NOT_FIND, ZERO_ADDR};

elrond_wasm::derive_imports!();

use elrond_wasm::types::heap::Vec;


// pub fn build_dealer(dealer_addr: u64) -> Dealer {
//      return Dealer {
//           state: 0,
//           addr: dealer_addr,
//           miners: Vec::new(),
//           markups:Vec::new(),
//           max_markups:Vec::new(),
//           tokens:Vec::new()
//      }
// }

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct Dealer {
     pub state:u8,                 //State
     pub addr:u64,                 //Adresse
     pub miners:Vec<u64>,          //Adresses des mineurs autorisés
     pub markups:Vec<u16>,          //syntaxe (<ids>,markup) Markup à appliqué pour les token de start à end
     pub max_markups:Vec<u16>,
     pub tokens:Vec<u64>
}

impl Dealer {
     pub fn new(dealer_addr:u64) -> Self {
          return Dealer {
               state: 1,
               addr: dealer_addr,
               miners: Vec::new(),
               markups:Vec::new(),
               max_markups:Vec::new(),
               tokens:Vec::new()
          }
     }


     pub fn get_idx(&self,id:u64) -> usize {
          return self.tokens.iter().position(|&x|x==id).unwrap_or(NOT_FIND);
     }

     pub fn find_markup(&self,token_idx:u64) -> (u16,u16) {
          let pos=self.get_idx(token_idx);
          if pos!=NOT_FIND {
               return (self.markups[pos],self.max_markups[pos]);
          }
          return (0,0);
     }

     pub fn is_zero(&self) -> bool {
          return self.addr==ZERO_ADDR;
     }



     pub fn add_token(&mut self, token_id:u64, markup:u16,max_markup:u16) -> bool {
          if self.tokens.contains(&token_id) {return false;}
          self.tokens.push(token_id);
          self.markups.push(markup);
          self.max_markups.push(max_markup);
          return true;
     }




     pub fn set_markup(&mut self,token_ids:Vec<u64>,new_markup:u16,max_markup:u16) -> bool {
          for id in token_ids {
               let position=self.get_idx(id);
               if position!=NOT_FIND {
                    if new_markup<self.max_markups[position] {
                         self.markups[position]=new_markup;
                    } else {
                         return false;
                    }
               } else {
                    self.add_token(id,new_markup,max_markup); //TODO: a corriger sur le max_markup
               }
          }
          return true;
     }

}






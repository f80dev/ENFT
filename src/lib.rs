//GNU GENERAL PUBLIC LICENSE - Version 3, 29 June 2007
//Auteur: Herve Hoareau

#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(unused_attributes)]
#![allow(non_snake_case)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod token;
mod dealer;
use token::Token;
use dealer::Dealer;


#[elrond_wasm_derive::contract(ENonFungibleTokensImpl)]
pub trait ENonFungibleTokens {

	//Initialisation du SC
	#[init]
	fn init(&self) {
		let owner = self.blockchain().get_caller();
		self.set_owner(&owner);
		self.set_total_minted(0); //Aucun token miné
		self.set_dealer_count(0); //Aucun distributeur
	}


	// fn decrypt(&self,secret: &Vec<u8>) -> Vec<u8> {
	// 	//TODO implémenter ici un fonction de décryptage
	// 	return secret;
	// }


	/// Creates new tokens and sets their ownership to the specified account.
	/// Only the contract owner may call this function.
	#[payable("EGLD")]
	#[endpoint]
	fn mint(&self,
			#[payment] payment: BigUint,
			count: u64,
			new_token_title: &Vec<u8>,
			new_token_description: &Vec<u8>,
			secret: &Vec<u8>,
			initial_price: u32,
			min_markup:u16,
			max_markup:u16,
			properties:u8,
			miner_ratio:u16,
			gift:u16,
			money:TokenIdentifier
	) -> SCResult<u64> {
		// let dec_secret=self.decrypt(secret);

		let caller=self.blockchain().get_caller();
		require!(count>0,"E01: At least one token must be mined");
		require!(new_token_title.len()+new_token_description.len() > 0,"E02: Title & description can't be empty together");
		require!(min_markup <= max_markup,"E03: L'interval de commission est incorrect");
		//La limite du miner_ratio est à 10000 car on multiplie par 100 pour autoriser un pourcentage à 2 décimal
		require!(miner_ratio<=10000,"E04: La part du mineur doit etre entre 0 et 100");
		require!(payment>=BigUint::from(count*gift as u64),"E05: Transfert de fond insuffisant pour le token");

		let token_id=self.perform_mint(count,caller,new_token_title,new_token_description,secret,initial_price,min_markup,max_markup,properties,miner_ratio,gift,&money);

		Ok(token_id)
	}




	/// Approves an account to transfer the token on behalf of its owner.<br>
	/// Only the owner of the token may call this function.
	#[endpoint]
	fn approve(&self, token_id: u64, approved_address: Address) -> SCResult<()> {
		let token=self.get_token(token_id);
		require!(token_id < self.get_total_minted(), "E06: Token does not exist!");
		require!(self.blockchain().get_caller() == token.owner ,"E07: Only the token owner can approve!");

		self.set_approval(token_id, &approved_address);

		Ok(())
	}



	/// Revokes approval for the token.<br>  
	/// Only the owner of the token may call this function.
	#[endpoint]
	fn revoke(&self, token_id: u64) -> SCResult<()> {
		require!(token_id < self.get_total_minted(), "E08: Token does not exist!");

		let token=self.get_token(token_id);
		require!(self.blockchain().get_caller() == token.owner,"E09: Only the token owner can revoke approval!");

		if !self.approval_is_empty(token_id) {
			//TODO: on considère les approuvés comme des distributeurs, on doit donc supprimer le distributeur
			self.perform_revoke_approval(token_id);
		}

		Ok(())
	}





	/// Transfer ownership of the token to a new account.
	#[endpoint]
	fn transfer(&self, token_id: u64, to: Address) -> SCResult<()> {
		require!(token_id < self.get_total_minted(), "E12: Token does not exist!");
		let mut token=self.get_token(token_id);

		//Le mineur peut avoir limité la possibilité de transfert du token à sa création
		require!(token.properties & 0b00000001 > 0,"E13: Ce token ne peut être offert");

		let caller = self.blockchain().get_caller();

		if caller == token.owner {
			token.owner=to;
			self.set_token(token_id,&token);
			//self.perform_transfer(token_id, &token.owner, &to);
			return Ok(());
		} else if !self.approval_is_empty(token_id) {
			//TODO code à conformer à ENFT
			let approved_address = self.get_approval(token_id);

			if caller == approved_address {
				token.owner=to;
				self.set_token(token_id,&token);

				//self.perform_transfer(token_id, &token.owner, &to);
				return Ok(());
			}
		}

		sc_error!("E14: Only the owner or the approved account may transfer the token!")
	}

	
	// Méthode privée utilisé pour effectivement créer le token
	//count permet de miner plusieurs tokens identique avec un seul appels
	fn perform_mint(&self,
					count:u64,
					new_token_owner: Address,
					new_token_title: &Vec<u8>,
					new_token_description: &Vec<u8>,
					secret: &Vec<u8>,
					new_token_price: u32,
					min_markup: u16,max_markup: u16,
					properties:u8,
					miner_ratio:u16,
					gift:u16,
					money: &TokenIdentifier) -> u64 {
		let new_owner_current_total = self.get_token_count(&new_token_owner);
		let total_minted = self.get_total_minted();
		let first_new_id = total_minted;
		let last_new_id = total_minted + count;

		for id in first_new_id..last_new_id {
			let token = Token {
				owner:new_token_owner.clone(),
				miner:new_token_owner.clone(),
				price:new_token_price.clone(),
				gift:gift,
				title:new_token_title.to_vec(),
				description:new_token_description.to_vec(),
				secret:secret.to_vec(),
				state:0 as u8,
				min_markup:min_markup,
				max_markup:max_markup,
				dealer_ids:Vec::new(),
				dealer_markup:Vec::new(),
				properties:properties,
				miner_ratio:miner_ratio,
				money:money.clone()
			};

			self.set_token(id, &token);
		}

		self.set_total_minted(total_minted + count);
		self.set_token_count(&new_token_owner, new_owner_current_total + count);
		return last_new_id;
	}





	fn perform_revoke_approval(&self, token_id: u64) {
		// clear at key "''approval|token_id"
		self.clear_approval(token_id);
	}


	fn perform_burn(self,token_id: u64,token: &mut Token) -> bool {

		if token.gift>0 {
			self.send().direct_egld(
				&token.miner,
				&BigUint::from(token.gift as u64*10000000000000000),
				b"Miner refund"
			);
		}

		token.miner=Address::zero();
		token.owner=Address::zero();
		self.set_token(token_id,&token);

		return true;
	}


	//Détruit un token en lui affectant l'adresse 0x0 comme propriétaire et mineur
	#[endpoint]
	fn burn(&self, token_id: u64) -> SCResult<()> {
		require!(token_id < self.get_total_minted(), "E15: Token does not exist!");

		let caller = self.blockchain().get_caller();
		let mut token = self.get_token(token_id);
		require!(caller == token.owner || caller == token.miner,"E16: Only the owner account can burn token!");

		self.perform_burn(token_id, &mut token);

		return Ok(());

	}

	fn send_money(&self,token:&Token,dest:&Address,amount:u64,comment:&[u8]) {
		if token.money.is_egld() {
			self.send().direct_egld(dest,&BigUint::from(amount*10000000000000000),comment);
		} else {
			self.send().direct(dest, &token.money, &BigUint::from(amount*10000000000000000), comment);
		}
	}



	//Retourne le contenu de la propriété secret du token en échange d'une vérification
	//que l'appelant est bien propriétaire du token
	#[endpoint]
	fn open(&self, token_id: u64) -> SCResult<Vec<u8>> {
		require!(token_id < self.get_total_minted(), "Token does not exist!");
		let mut token=self.get_token(token_id);

		let caller = self.blockchain().get_caller();
		require!(caller == token.owner,"E10: Seul le propriétaire peut ouvrir le token");
		require!(token.secret.len()>0,"E11: Ce token ne contient pas de secret");

		//let secret=mc.decrypt_base64_to_string(&token.secret).unwrap();
		//TODO: mettre en place le décryptage du secret
		//secret=self.decrypt(&secret);

		let secret=token.secret.clone();
		//https://docs.rs/openssl/0.10.32/openssl/rsa/index.html
		//let secret=v3::decrypt("secret",&enc_data);

		if token.gift>0 {
			self.send_money(&token,&token.owner,token.gift as u64,b"Owner pay");
			token.gift=0;
			self.set_token(token_id,&token);
		}

		if token.properties & 0b00001000>0 {
			self.perform_burn(token_id,&mut token);
		}

		return Ok(secret);
	}




	//Permet la mise en vente ou le retrait de la vente d'un token
	//Seul le propriétaire du token peut le remettre en vente
	#[endpoint]
	fn setstate(&self,  token_id: u64,new_state:u8) -> SCResult<()> {
		let mut token = self.get_token(token_id);
		let caller=self.blockchain().get_caller();

		require!(token_id < self.get_total_minted(), "E19: Token does not exist!");
		require!(token.owner == caller,"E17: Only token owner change state");
		require!(token.properties & 0b00000010>0,"E18: Revente interdite");

		token.state=new_state;
		self.set_token(token_id,&token);

		Ok(())
	}

	//Recherche un dealer par son adresse
	//retourne dealer_count si on a pas trouvé le dealer
	fn find_dealer_by_addr(&self,dealer_addr: &Address) -> u64 {
		let total=self.get_dealer_count();
		for i in 0..total {
			let dealer=self.get_dealer(i);
			if &dealer.addr==dealer_addr {
				return i;
			}
		}
		return total;
	}


	//Recherche un dealer par son adresse dans un token
	fn find_dealer_in_token(&self,dealer_addr: &Address,token:&Token) -> usize {
		let mut rc=token.dealer_ids.len();
		if dealer_addr != &Address::zero() {
			let addrs=self.get_dealer_addresses_for_token(&token);
			rc=addrs.iter().position(|x| x == dealer_addr).unwrap_or(token.dealer_ids.len());
		}
		return rc;
	}


	//Ajouter un miner approuvé à un dealer
	#[endpoint]
	fn add_miner(&self,  miner_addr: &Address,ipfs_token:BigUint) -> SCResult<()> {
		let dealer_id=self.find_dealer_by_addr(&self.blockchain().get_caller());
		require!(dealer_id < self.get_dealer_count(), "Dealer not listed");

		let mut dealer=self.get_dealer(dealer_id);
		dealer.miners.push(miner_addr.clone());
		self.set_dealer(dealer_id,&dealer);

		self.ipfs_map().insert(miner_addr.clone(),ipfs_token);

		Ok(())
	}


	//Supprimer un miner approuvé à un dealer
	#[endpoint]
	fn del_miner(&self,  miner_addr: &Address) -> SCResult<()> {
		let dealer_id=self.find_dealer_by_addr(&self.blockchain().get_caller());
		require!(dealer_id < self.get_dealer_count(), "Dealer not listed");

		let mut dealer=self.get_dealer(dealer_id);

		let mut idx=0;
		for miner in dealer.miners.iter() {
			if miner == miner_addr {
				dealer.miners.remove(idx);
				self.set_dealer(dealer_id,&dealer);
				break;
			}
			idx=idx+1;
		}

		Ok(())
	}


	#[endpoint]
	fn dealer_state(&self,  state: u8) -> SCResult<()> {
		let addr = self.blockchain().get_caller();
		let idx = self.find_dealer_by_addr(&addr);

		require!(idx<self.get_dealer_count(),"Dealer not listed");
		let mut dealer=self.get_dealer(idx);

		dealer.state=state;
		self.set_dealer(idx,&dealer);
		return Ok(());
	}


	//Ajout un nouveau distributeur
	//state=0 open / 1 close
	#[endpoint]
	fn new_dealer(&self,  ipfs: &Vec<u8>) -> SCResult<u64> {
		let addr=self.blockchain().get_caller();
		let idx=self.find_dealer_by_addr(&addr);
		if idx == self.get_dealer_count() {
			let dealer = Dealer {
				ipfs: ipfs.to_vec(),
				state: 0,
				addr: addr.clone(),
				miners: Vec::new()
			};
			self.set_dealer(idx,&dealer);
			self.set_dealer_count(idx+1);
		}
		Ok(idx)
	}



	#[view(is_miner)]
	fn is_miner(&self,  miner_addr: Address) -> bool {
		return self.ipfs_map().contains_key(&miner_addr);
	}



		//Retourne la liste des mineurs approuvé par un distributeurs (separateur de liste : 000000)
	#[view(miners)]
	fn miners(&self,  dealer_addr: Address) -> Vec<u8> {

		let mut rc=Vec::new();
		let idx=self.find_dealer_by_addr(&dealer_addr);
		if idx==self.get_dealer_count() {
			return rc;
		}

		let dealer=self.get_dealer(idx);
		for miner in dealer.miners.iter() {
			rc.append(&mut miner.to_vec());
			let ipfs=self.ipfs_map().get(miner).unwrap();
			rc.append(&mut ipfs.to_bytes_be().to_vec());
			rc.push(0);
			rc.push(0);
			rc.push(0);
		}

		return rc;
	}


	//retourne l'ensemble des distributeurs référencés si l'adresse est 0x0 ou les distributeurs d'un mineur
	#[view(dealers)]
	fn dealers(&self,filter_miner:Address) -> Vec<u8> {
		let mut rc=Vec::new();

		for idx in 0..self.get_dealer_count() {
			let dealer=self.get_dealer(idx);
			if filter_miner==Address::zero() || dealer.miners.contains(&filter_miner) {
				rc.append(&mut dealer.addr.to_vec());
				rc.push(dealer.state);
				rc.append(&mut dealer.ipfs.to_vec());
				rc.push(0);
				rc.push(0);
				rc.push(0);
			}
		}

		return rc;
	}



		//Permet d'ajouter un distributeur à la liste des distributeurs du token
	//Cette fonction est réservé au propriétaire du token
	#[endpoint]
	fn add_dealer(&self,  token_id: u64, addr: Address) -> SCResult<()> {
		let caller=self.blockchain().get_caller();
		let mut token = self.get_token(token_id);

		require!(token_id < self.get_total_minted(), "E20: Token does not exist!");
		require!(token.owner == caller,"E21: Only token owner can add dealer");

		let dealer_id = self.find_dealer_by_addr(&addr);
		require!(dealer_id < self.get_dealer_count() ,"Distributeur non reference");

		let dealer=self.get_dealer(dealer_id);

		//Recherche du mineur du token dans la whitelist du dealer
		for miner_addr in dealer.miners.iter() {
			if miner_addr==&token.miner {
				//On ajoute le nouveau dealer au token
				token.dealer_ids.push(dealer_id);
				token.dealer_markup.push(0u16);
				self.set_token(token_id,&token);
				return Ok(())
			}
		}

		sc_error!("le miner du token n'est pas autorisé par le dealer")
	}


	//efface l'ensemble des distributeurs
	#[endpoint]
	fn clear_dealer(&self,  token_id: u64) -> SCResult<()> {
		let mut token = self.get_token(token_id);
		let caller=self.blockchain().get_caller();

		require!(token_id < self.get_total_minted(), "E22: Token does not exist!");
		require!(token.owner == caller,"E23: Only token owner can remove dealer");

		token.dealer_ids=Vec::new();
		token.dealer_markup=Vec::new();

		self.set_token(token_id,&token);

		return Ok(())
	}



	//Modifier le prix (dans la fourchette initialement défini par le mineur du token)
	//Seul les distributeurs peuvent modifier le prix
	#[endpoint]
	fn price(&self, token_id: u64, markup: u16) -> SCResult<()> {
		require!(token_id < self.get_total_minted(), "E24: Token does not exist!");
		let mut token = self.get_token(token_id);

		let caller = self.blockchain().get_caller();

		let addrs=self.get_dealer_addresses_for_token(&token);
		let idx = addrs.iter().position(|x| x == &caller).unwrap_or(1000);

		require!(idx<1000, "E25: Modif de prix uniquement pour les distributeurs");
		require!(markup <= token.max_markup,"E26: Vous ne pouvez pas augmenter autant le prix");
		require!(markup >= token.min_markup,"E27: Vous ne pouvez pas réduire autant le prix");

		token.dealer_markup[idx] = markup;
		self.set_token(token_id,&token);

		return Ok(())
	}



	//Fonction d'achat d'un token
	//token_id: désigne le token à acheter
	//dealer: déclare le vendeur qui à fait la vente. Cela permet au système de récupéré le prix avec la commission et de procéder au reversement
	#[payable("*")]
	#[endpoint]
	fn buy(&self, #[payment] payment: BigUint,#[payment_token] pay_token: TokenIdentifier, token_id: u64,dealer:Address) -> SCResult<()> {

		require!(token_id < self.get_total_minted(), "E28: Ce token n'existe pas");
		let mut token = self.get_token(token_id);

		let caller=self.blockchain().get_caller();
		require!(token.owner != caller,"E29: Ce token vous appartient déjà");
		require!(token.state == 0,"E30: Ce token n'est pas en vente");

		let addrs=self.get_dealer_addresses_for_token(&token);
		let idx = addrs.iter().position(|x| x == &dealer).unwrap_or(1000);

		let mut payment_for_dealer=0u64;
		if idx<1000 {
			payment_for_dealer=10000000000000000*token.dealer_markup[idx] as u64;
		}

		require!(token.properties & 0b00000100>0 || dealer!=Address::zero() ,"E31: La vente directe n'est pas autorisé");
		require!(dealer==Address::zero() || idx<1000 ,"E32: Le revendeur n'est pas autorisé");
		require!(payment >= BigUint::from(payment_for_dealer+100000000000000*token.price.clone() as u64),"E33: Paiement inferieur au prix du token");

		//Versement au vendeur
		let temp:vec<u8>=payment.to_bytes_be();
		let mut array: [u8; 8] = [0,0,0,0,0,0,0,0];
		for i in 0..8 {
			array[i]=temp[i];
		}
		let payment_for_owner=u64::from_be_bytes(array)-payment_for_dealer;

		if dealer!=Address::zero() && payment_for_dealer>0 {
			//On retribue le mineur sur la commission du distributeur
			if token.miner_ratio>0 {
				let payment_for_miner=1000000000000*token.dealer_markup[idx] as u64*token.miner_ratio as u64;
				self.send_money(&token,&token.miner,payment_for_miner,b"miner pay");
				payment_for_dealer=payment_for_dealer-payment_for_miner;
			}

			//Transaction issue d'un revendeur
			self.send_money(&token,&dealer,payment_for_dealer,b"dealer pay");
		}

		if payment_for_owner>0 {
			self.send_money(&token,&token.owner,payment_for_owner,b"owner pay");
		}

		token.state=1;//Le token n'est plus a vendre
		token.owner=caller; //On change le propriétaire
		self.set_token(token_id,&token);

		return Ok(());
	}




	fn get_dealer_addresses_for_token(&self,token: &Token) -> Vec<Address> {
		let mut rc=Vec::new();
		for i in 0..token.dealer_ids.len(){
			let dealer=self.get_dealer(token.dealer_ids[i] as u64);
			rc.push(dealer.addr);
		}
		return rc;
	}



	//Récupérer l'ensemble des tokens en appliquant les filtres sauf si celui est à la valeur 0x0
	//seller: uniquement les tokens dont "seller" fait parti des distributeurs déclarés
	//owner: uniquement les tokens dont le propriétaire est "owner"
	//miner: uniquement les tokens fabriqués par "miner"
	#[view(tokens)]
	fn get_tokens(&self,seller_filter: Address,owner_filter: Address, miner_filter: Address) -> Vec<Vec<u8>> {
		let mut rc=Vec::new();

		let total_minted = self.get_total_minted();

		for i in 0..total_minted {
			let mut token=self.get_token(i);

			let idx = self.find_dealer_in_token(&seller_filter,&token);

			if (owner_filter == Address::zero() || owner_filter == token.owner)
				&& (miner_filter == Address::zero() || miner_filter == token.miner)
				&& (seller_filter == Address::zero() || idx < token.dealer_ids.len() ) {

				let mut item:Vec<u8>=Vec::new();

				//On commence par inscrire la taille de token_price & title dont les tailles dépendent du contenu
				//doc sur le conversion :https://docs.rs/elrond-wasm/0.10.3/elrond_wasm/
				item.append(&mut token.title.len().to_be_bytes().to_vec());
				item.append(&mut token.description.len().to_be_bytes().to_vec());

				//Puis on ajoute l'ensemble des informations d'un token
				//dans un vecteur d'octets
				let mut price=token.price;
				let mut markup=0u16;
				if idx< token.dealer_ids.len()  {
					price=price+100*token.dealer_markup[idx] as u32;
					markup=token.dealer_markup[idx];
				}

				let mut has_secret=0u8;
				if token.secret.len()>1 || token.gift>0 {
					has_secret=1u8;
				}

				item.append(&mut price.to_be_bytes().to_vec());
				item.append(&mut token.owner.to_vec());
				item.push(has_secret);
				item.push(token.state);
				item.push(token.properties);
				item.append(&mut token.min_markup.to_be_bytes().to_vec());
				item.append(&mut token.max_markup.to_be_bytes().to_vec());
				item.append(&mut markup.to_be_bytes().to_vec());
				item.append(&mut token.miner_ratio.to_be_bytes().to_vec());
				item.append(&mut token.miner.to_vec());
				item.append(&mut i.to_be_bytes().to_vec());
				item.append(&mut token.title);
				item.append(&mut token.description);

				rc.push(item);
			}

		}
		return rc;
	}




	#[view(contractOwner)]
	#[storage_get("owner")]
	fn get_owner(&self) -> Address;
	#[storage_set("owner")]
	fn set_owner(&self, owner: &Address);


	// #[storage_get("private_key")]
	// fn get_private_key(&self) -> RSAPrivateKey;
	// #[storage_set("owner")]
	// fn set_private_key(&self, key: &RSAPrivateKey);


	// Fonctions utilisées pour les NFT
	// #[view(tokenOwner)]
	// #[storage_get("tokenOwner")]
	// fn get_token_owner(&self, token_id: u64) -> Address;
	// #[storage_set("tokenOwner")]
	// fn set_token_owner(&self, token_id: u64, owner: &Address);


	//Retourne le nombre total de token minés
	#[view(totalMinted)]
	#[storage_get("totalMinted")]
	fn get_total_minted(&self) -> u64;
	#[storage_set("totalMinted")]
	fn set_total_minted(&self, total_minted: u64);


	#[storage_mapper("ipfs")]
	fn ipfs_map(&self) -> MapMapper<Self::Storage, Address, BigUint>;


	#[view(dealerCount)]
	#[storage_get("dealerCount")]
	fn get_dealer_count(&self) -> u64;
	#[storage_set("dealerCount")]
	fn set_dealer_count(&self, token_count: u64);


	#[view(tokenCount)]
	#[storage_get("tokenCount")]
	fn get_token_count(&self, owner: &Address) -> u64;
	#[storage_set("tokenCount")]
	fn set_token_count(&self, owner: &Address, token_count: u64);


	//Récupération d'un token
	#[view(getToken)]
	#[storage_get("token")]
	fn get_token(&self,  token_id: u64) -> Token;
	#[storage_set("token")]
	fn set_token(&self, token_id: u64, token: &Token);



	//Information sur les mineurs / créateurs
	#[view(getMinerInfos)]
	#[storage_get("minerInfos")]
	fn get_miner_infos(&self,  miner: &Address) -> Vec<u8>;
	#[storage_set("minerInfos")]
	fn set_miner_infos(&self, miner: &Address, infos: Vec<u8>);



	//Récupération d'un dealer
	#[view(getDealer)]
	#[storage_get("dealer")]
	fn get_dealer(&self,  dealer_id: u64) -> Dealer;
	#[storage_set("dealer")]
	fn set_dealer(&self, dealer_id: u64, dealer: &Dealer);



	//Fonction d'approbation pour maintient de compatibilité avec les NFT
	#[storage_is_empty("approval")]
	fn approval_is_empty(&self, token_id: u64) -> bool;
	#[view(approval)]
	#[storage_get("approval")]
	fn get_approval(&self, token_id: u64) -> Address;
	#[storage_set("approval")]
	fn set_approval(&self, token_id: u64, approved_address: &Address);
	#[storage_clear("approval")]
	fn clear_approval(&self, token_id: u64);


}

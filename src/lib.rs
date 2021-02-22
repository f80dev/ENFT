#![no_std]
#![allow(clippy::too_many_arguments)]

imports!();

mod token;
use token::Token;

#[elrond_wasm_derive::contract(ENonFungibleTokensImpl)]
pub trait ENonFungibleTokens {
	#[init]
	fn init(&self) {
		let owner = self.get_caller();
		self.set_owner(&owner);
		self.set_total_minted(0);
	}

	// endpoints

	/// Creates new tokens and sets their ownership to the specified account.
	/// Only the contract owner may call this function.
	#[endpoint]
	fn mint(&self,
			count: u64,
			new_token_uri: &Vec<u8>,
			secret: &Vec<u8>,
			new_token_price: BigUint,
			min_markup:u16,
			max_markup:u16,
			owner_seller:u8,
			miner_ratio:u16
	) -> SCResult<u64> {
		let caller=self.get_caller();
		require!(count>0,"At least one token must be mined");
		require!(new_token_uri.len() > 0,"URI can't be empty");

		let token_id=self.perform_mint(count,caller,new_token_uri,secret,new_token_price,min_markup,max_markup,owner_seller,miner_ratio);

		Ok(token_id)
	}




	/// Approves an account to transfer the token on behalf of its owner.<br>
	/// Only the owner of the token may call this function.
	#[endpoint]
	fn approve(&self, token_id: u64, approved_address: Address) -> SCResult<()> {
		let token=self.get_token(token_id);
		require!(token_id < self.get_total_minted(), "Token does not exist!");
		require!(self.get_caller() == token.owner ,"Only the token owner can approve!");

		self.set_approval(token_id, &approved_address);

		Ok(())
	}



	/// Revokes approval for the token.<br>  
	/// Only the owner of the token may call this function.
	#[endpoint]
	fn revoke(&self, token_id: u64) -> SCResult<()> {
		require!(token_id < self.get_total_minted(), "Token does not exist!");

		let token=self.get_token(token_id);
		require!(self.get_caller() == token.owner,"Only the token owner can revoke approval!");

		if !self.approval_is_empty(token_id) {
			self.perform_revoke_approval(token_id);
		}

		Ok(())
	}
	

	// fn decrypt(&self, encrypted:&Vec<u8>) -> Vec<u8> {
	// 	let a:u8=12;
	// 	for u in encrypted {
	// 		u = &(u ^ a);
	// 	}
	// 	return encrypted.to_vec();
	// }



	#[endpoint]
	fn open(&self, token_id: u64) -> SCResult<Vec<u8>> {
		require!(token_id < self.get_total_minted(), "Token does not exist!");

		let caller = self.get_caller();

		let token=self.get_token(token_id);

		let secret=token.secret.to_vec();
		//TODO: mettre en place le décryptage du secret
		//secret=self.decrypt(&secret);


		if caller == token.owner {
			return Ok(secret);
		}

		sc_error!("You are not the owner of this token")
	}


	/// Transfer ownership of the token to a new account.
	#[endpoint]
	fn transfer(&self, token_id: u64, to: Address) -> SCResult<()> {
		require!(token_id < self.get_total_minted(), "Token does not exist!");
		let mut token=self.get_token(token_id);

		//Le mineur peut avoir limité la possibilité de transfert du token à sa création
		require!(token.seller_owner==1 || token.seller_owner==3,"Ce token ne peut être offert");

		let caller = self.get_caller();

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

		sc_error!("Only the owner or the approved account may transfer the token!")
	}




	// Méthode privée utilisé pour effectivement créer le token
	//count permet de miner plusieurs tokens identique avec un seul appels
	fn perform_mint(&self, count:u64,
					new_token_owner: Address,
					new_token_uri: &Vec<u8>,
					secret: &Vec<u8>,
					new_token_price: BigUint,
					min_markup: u16,max_markup: u16,
					owner_seller:u8,
					miner_ratio:u16) -> u64 {
		let new_owner_current_total = self.get_token_count(&new_token_owner);
		let total_minted = self.get_total_minted();
		let first_new_id = total_minted;
		let last_new_id = total_minted + count;

		for id in first_new_id..last_new_id {
			let token = Token {
				owner:new_token_owner.clone(),
				miner:new_token_owner.clone(),
				price:new_token_price.clone(),
				uri:new_token_uri.to_vec(),
				secret:secret.to_vec(),
				state:0 as u8,
				min_markup:min_markup,
				max_markup:max_markup,
				dealer_addr:Vec::new(),
				dealer_markup:Vec::new(),
				seller_owner:owner_seller,
				miner_ratio:miner_ratio,
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


	//Fonction obsolete appelée dans le cadre de la norme ERC721
	// fn perform_transfer(&self, token_id: u64, from: &Address, to: &Address) {
	// 	let prev_owner_token_count = self.get_token_count(from);
	// 	let new_owner_token_count = self.get_token_count(to);
	//
	// 	// new ownership revokes approvals by previous owner
	// 	self.perform_revoke_approval(token_id);
	//
	// 	self.set_token_count(from, prev_owner_token_count - 1);
	// 	self.set_token_count(to, new_owner_token_count + 1);
	//
	// 	let mut token=self.get_token(token_id);
	// 	token.owner=to.clone();
	// 	self.set_token(token_id, &token);
	//
	// }


	//Détruit un token en lui affectant l'adresse 0x0 comme propriétaire et mineur
	#[endpoint]
	fn burn(&self, token_id: u64) -> SCResult<()> {
		require!(token_id < self.get_total_minted(), "Token does not exist!");

		let caller = self.get_caller();
		let mut token = self.get_token(token_id);

		if caller == token.owner || caller == token.miner {
			token.miner=Address::zero();
			token.owner=Address::zero();
			self.set_token(token_id,&token);
			return Ok(());
		}

		sc_error!("Only the owner account can burn token!")
	}


	//Permet la mise en vente ou le retrait de la vente d'un token
	//Seul le propriétaire du token peut le remettre en vente
	#[endpoint]
	fn setstate(&self,  token_id: u64,new_state:u8) -> SCResult<()> {
		let mut token = self.get_token(token_id);
		let caller=self.get_caller();

		require!(token_id < self.get_total_minted(), "Token does not exist!");
		require!(token.owner == caller,"Only token owner or reseller can change the state");
		require!(token.seller_owner==2 || token.seller_owner==3,"Le créateur du token n'autorise pas le propriétaire à le vendre");

		token.state=new_state;
		self.set_token(token_id,&token);

		Ok(())
	}



	//Permet d'ajouter un distributeur à la liste des distributeurs du token
	//Cette fonction est réservé au propriétaire du token
	#[endpoint]
	fn add_dealer(&self,  token_id: u64, addr: Address) -> SCResult<()> {
		let caller=self.get_caller();
		let mut token = self.get_token(token_id);

		require!(token_id < self.get_total_minted(), "Token does not exist!");
		require!(token.owner == caller,"Only token owner can add dealer");
		//TODO: ajouter un require pour s'assurer que le developpeur n'est pas déjà dans la liste


		token.dealer_addr.push(addr);
		token.dealer_markup.push(0u16);
		self.set_token(token_id,&token);

		Ok(())
	}


	//efface l'ensemble des distributeurs
	#[endpoint]
	fn clear_dealer(&self,  token_id: u64) -> SCResult<()> {
		let mut token = self.get_token(token_id);
		let caller=self.get_caller();

		require!(token_id < self.get_total_minted(), "Token does not exist!");
		require!(token.owner == caller,"Only token owner can clear the dealer list");

		token.dealer_addr=Vec::new();
		token.dealer_markup=Vec::new();

		self.set_token(token_id,&token);

		Ok(())
	}



	//Modifier le prix (dans la fourchette initialement défini par le mineur du token)
	//Seul les distributeurs peuvent modifier le prix
	#[endpoint]
	fn price(&self, token_id: u64, markup: u16) -> SCResult<()> {
		require!(token_id < self.get_total_minted(), "Token does not exist!");
		let mut token = self.get_token(token_id);

		let caller = self.get_caller();

		let idx = token.dealer_addr.iter().position(|x| x == &caller).unwrap_or(1000);

		require!(idx<1000, "Seul les distributeurx peuvent modifier le prix");

		require!(markup <= token.max_markup,"Vous ne pouvez pas augmenter autant le prix");
		require!(markup >= token.min_markup,"Vous ne pouvez pas réduire autant le prix");

		token.dealer_markup[idx] = markup;
		self.set_token(token_id,&token);

		return Ok(())
	}



	//Fonction d'achat d'un token
	//token_id: désigne le token à acheter
	//dealer: déclare le vendeur qui à fait la vente. Cela permet au système de récupéré le prix avec la commission et de procéder au reversement
	#[payable("EGLD")]
	#[endpoint]
	fn buy(&self, #[payment] payment: BigUint, token_id: u64,dealer:Address) -> SCResult<&str> {

		require!(token_id < self.get_total_minted(), "Token does not exist!");

		let mut token = self.get_token(token_id);
		let caller=self.get_caller();

		require!(token.owner != caller,"Ce token vous appartient déjà");
		require!(token.state == 0,"Ce token n'est pas en vente");

		let idx = token.dealer_addr.iter().position(|x| x == &dealer).unwrap_or(1000);
		let mut payment_for_dealer=0u64;
		if idx<1000 {
			payment_for_dealer=10000000000000000*token.dealer_markup[idx] as u64;
		}

		require!(dealer==Address::zero() || idx<1000 ,"Le revendeur n'est pas autorisé");
		require!(payment >= token.price.clone()+BigUint::from(payment_for_dealer),"Montant inferieur au prix");

		//Versement au vendeur
		let payment_for_owner=payment.clone()-BigUint::from(payment_for_dealer);


		if dealer!=Address::zero() && payment_for_dealer>0 {
			//On retribue le mineur sur la commission du distributeur
			if token.miner_ratio>0 {
				let payment_for_miner=payment_for_dealer/(10000u64/token.miner_ratio as u64);
				self.send().direct_egld(
					&token.owner,
					&BigUint::from(payment_for_miner),
					b"Reglement du miner"
				);
				payment_for_dealer=payment_for_dealer-payment_for_miner;
			}

			//Transaction issue d'un revendeur
			self.send().direct_egld(
				&token.dealer_addr[idx],
				&BigUint::from(payment_for_dealer),
				b"Reglement du dealer"
			);

		}

		if payment_for_owner>0 {
			self.send().direct_egld(
				&token.owner,
				&payment_for_owner,
				b"Reglement du owner"
			);
		}


		token.state=1;//Le token n'est plus a vendre
		token.owner=caller; //On change le propriétaire
		self.set_token(token_id,&token);


		return Ok(&"Achat terminé");
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

			let idx = token.dealer_addr.iter().position(|x| x == &seller_filter).unwrap_or(1000);

			if (owner_filter == Address::zero() || owner_filter == token.owner)
				&& (miner_filter == Address::zero() || miner_filter == token.miner)
				&& (seller_filter == Address::zero() || idx<1000) {
				let mut item:Vec<u8>=Vec::new();

				//On commence par inscrire la taille de token_price & uri dont les tailles dépendent du contenu
				//doc sur le conversion :https://docs.rs/elrond-wasm/0.10.3/elrond_wasm/
				item.append(&mut token.uri.len().to_be_bytes().to_vec());

				//Puis on ajoute l'ensemble des informations d'un token
				//dans un vecteur d'octets
				let mut price=token.price;
				if idx<1000 {
					price=price+BigUint::from(10000000000000000*token.dealer_markup[idx] as u64);
				}

				item.append(&mut price.to_bytes_be_pad_right(10).unwrap_or(Vec::new()));
				item.append(&mut token.owner.to_vec());
				item.push(token.state);
				item.push(token.seller_owner);
				item.append(&mut token.min_markup.to_be_bytes().to_vec());
				item.append(&mut token.max_markup.to_be_bytes().to_vec());
				item.append(&mut i.to_be_bytes().to_vec());
				item.append(&mut token.uri);

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



	#[view(tokenCount)]
	#[storage_get("tokenCount")]
	fn get_token_count(&self, owner: &Address) -> u64;
	#[storage_set("tokenCount")]
	fn set_token_count(&self, owner: &Address, token_count: u64);


	//Récupération d'un token
	#[view(getToken)]
	#[storage_get("token")]
	fn get_token(&self,  token_id: u64) -> Token<BigUint>;
	#[storage_set("token")]
	fn set_token(&self, token_id: u64, token: &Token<BigUint>);


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


	//Anciennes fonctions NFT, obsolètes dans la structure actuelle le miner étant stoker dans la structure du token
	// #[view(tokenMiner)]
	// #[storage_get("tokenMiner")]
	// fn get_token_miner(&self, token_id: u64) -> Address;
	// #[storage_set("tokenMiner")]
	// fn set_token_miner(&self, token_id: u64, miner_address: &Address);



}

//GNU GENERAL PUBLIC LICENSE - Version 3, 29 June 2007
//Auteur: Herve Hoareau

#![no_std]
#![feature(type_ascription)]

elrond_wasm::imports!();
use token::Token;
use dealer::Dealer;

mod token;
mod dealer;

const CONVERT_TO_GAS:u64 	=10000000000000000; //Facteur de conversion en gas

//const IS_MASTER:u8    	    =0b00000010; //est utilisé par un clone (dans ce cas là le token ne peut être brulé)
//const IS_CLONE:u8    	    =0b00000001; //Le token ne peut être possédé qu'une seul fois

//const ID_REQUIRED:u16		=0b1000000000000000;
const FOR_SALE:u16			=0b0100000000000000;
const ONE_WINNER:u16    	=0b0010000000000000; //Le token ne peut être possédé qu'une seul fois
const MINER_CAN_BURN:u16	=0b0001000000000000; //Le token ne peut être possédé qu'une seul fois
const UNIK:u16				=0b0000100000000000; //Le token ne peut être possédé qu'une seul fois
const SECRET_VOTE:u16		=0b0000010000000000;
//const VOTE:u16			=0b0000000100000000;
const RENT:u16				=0b0000000010000000; //A l'ouverture le contrat est restitué au créateur;
//const TRANSPARENT    		=0b0000000001000000;
//const FORCE_OPEN:u16		=0b0000000000100000;
const FIND_SECRET:u16		=0b0000000000010000;
const SELF_DESTRUCTION:u16	=0b0000000000001000;
const DIRECT_SELL:u16		=0b0000000000000100;
const CAN_RESELL:u16		=0b0000000000000010;
const CAN_TRANSFERT:u16		=0b0000000000000001;

//const MINER:u8 = 0;
//const OWNER:u8 = 1;
const NOT_FIND:usize = usize::MAX;
const ZERO_ADDR:u64 = 1;
//const MAX_U64:u64=4294967296;


#[elrond_wasm::contract]
pub trait ENonFungibleTokens
{

	#[view(tokens_map)]
	#[storage_mapper("tokens_map")]
	fn tokens_map(&self) -> VecMapper<Token>;


	//Récupération d'un token
	// #[view(getToken)]
	// #[storage_get("token")]
	// fn get_token(&self,  token_id: u64) -> Token;
	// #[storage_set("token")]
	// fn set_token(&self, token_id: u64, token: &Token);


	//Ajout d'une adresse dans le referentiel d'adresses si elle n'était pas encore présente
	//retourne la position de l'adresse
	#[view(set_addresses)]
	fn set_addresses(&self,new_addr: &ManagedAddress) -> u64 {
		let rc=self.addresses();
		let mut idx =rc.load_as_vec().iter().position(|r| r==new_addr).unwrap_or(NOT_FIND);
		if idx == NOT_FIND {
			idx = rc.len();
			self.addresses().push(new_addr);
		}
		return idx as u64;
	}



	#[view(get_idx_token_by_description)]
	fn get_idx_token_by_desc_and_addr(&self,description: &Vec<u8>,miner:&ManagedAddress) -> u64 {
		let idx_description=self.get_idx_str(&description) as u64;
		let idx_miner=self.get_idx_addresses(miner);
		let idx= self.tokens_map().load_as_vec().iter().position(|r| r.description==idx_description && r.miner == idx_miner).unwrap_or(NOT_FIND);
		return idx as u64;
	}

	//Retourne vrai si le owner (idx_owner) possede un token du miner_addr et meme description
	fn owner_has_required_token(&self,idx_owner:u64,idx_description:u64, idx_miner_addr: u64) -> bool {
		let idx= self.tokens_map().load_as_vec().iter().position(|t| t.description==idx_description && t.miner == idx_miner_addr && t.owner==idx_owner).unwrap_or(NOT_FIND);
		return idx != NOT_FIND;
	}


	fn get_addresses(&self,idx:u64) -> ManagedAddress {
		return self.addresses().get(idx as usize+1);
	}

	#[view(get_idx_addresses)]
	fn get_idx_addresses(&self,new_addr: &ManagedAddress) -> u64 {
		let idx= self.addresses().load_as_vec().iter().position(|r| r==new_addr).unwrap_or(NOT_FIND);
		return idx as u64;
	}

	//Retrouve l'index d'un dealer dans la liste des dealers
	#[view(get_idx_dealer)]
	fn get_idx_dealer(&self,idx_dealer_addr: u64) -> usize {
		let idx= self.dealers_map().load_as_vec().iter().position(|r:&Dealer| r.addr==idx_dealer_addr).unwrap_or(NOT_FIND);
		return idx+1;
	}

	//Gestion des chaines de caractères
	#[view(get_str)]
	fn get_str(&self,idx: u64) -> Vec<u8> {
		return self.strs().get(idx as usize+1);
	}

	#[view(get_idx_str)]
	fn get_idx_str(&self,vec: &Vec<u8>) -> usize {
		let idx =self.strs().load_as_vec().iter().position(|r| r == vec).unwrap_or(NOT_FIND);
		return idx;
	}


	fn set_str(&self,vec: &Vec<u8>) -> u64 {
		let mut rc=self.strs();
		let mut idx =rc.load_as_vec().iter().position(|r| r == vec).unwrap_or(NOT_FIND);
		if idx == NOT_FIND {
			idx = rc.len();
			rc.push(vec);
		}
		return idx as u64;
	}



	fn set_esdt(&self,new_token: &TokenIdentifier) -> u16 {
		let mut rc=self.esdt_map();
		let mut idx =rc.load_as_vec().iter().position(|r| r == new_token).unwrap_or(NOT_FIND);
		if idx == NOT_FIND {
			idx = rc.len();
			rc.push(&new_token);
		}
		return (idx+1) as u16; //Car la lecture du vecteur commence à 1
	}





	fn dealer_to_vec(&self,dealer: &Dealer) -> Vec<u8> {
		let mut rc =Vec::new();
		rc.append(&mut self.get_addresses(dealer.addr).to_address().to_vec());
		rc.push(dealer.state);
		return rc;
	}



	#[init]
	fn init(&self) {
		self.set_addresses(&self.blockchain().get_sc_address());
		self.set_addresses(&ManagedAddress::zero());

		self.esdt_map().push(&TokenIdentifier::egld());

		self.set_str(&Vec::from("Start"));

		self.dealers_map().push(&Dealer::new(ZERO_ADDR));

		self.tokens_map().push(&Token {
			owner:ZERO_ADDR,
			miner:ZERO_ADDR,
			creator: ZERO_ADDR,
			price:0,
			resp:0u8,
			gift:0,
			limit:0,
			description:0,
			secret:0,
			collection: 0,
			status: 0,
			properties:0,
			miner_ratio:0,	money: 0,
			deadline: 0,
			required:Vec::new(),
		});

	}


	// fn decrypt(&self,secret: &Vec<u8>) -> Vec<u8> {
	// 	//TODO implémenter ici un fonction de décryptage
	// 	return secret;
	// }

	#[endpoint(clone)]
	#[payable("EGLD")]
	fn clone(&self, ref_token_id: u64,count: u64,owner: ManagedAddress) -> SCResult<Vec<u64>> {
		let mut token=self.tokens_map().get(ref_token_id as usize);
		require!(token.miner==self.get_idx_addresses(&self.blockchain().get_caller()),"E88: Seul le miner peut cloner un token");

		//TODO: ajouter la vérification de l'autorisation de cloner

		let (payment, _pay_token)=self.call_value().payment_token_pair();
		let money=self.esdt_map().get(token.money as usize);
		if money.is_egld() && (token.properties & ONE_WINNER==0) && token.gift>0 {
			require!(payment == BigUint::from(token.gift as u64)*count,"E05: Transfert de fonds ne correspond pas au NFT");
		}

		let new_owner_addr=self.get_idx_addresses(&owner);
		if new_owner_addr!=ZERO_ADDR {
			token.owner=new_owner_addr;
		}

		let mut rc=Vec::new();
		for _i in 0..count {
			self.tokens_map().push(&token);
			rc.push(self.get_total_minted());
		}

		return Ok(rc) //On retourne le ref_token_id
	}


	/// Creates new tokens and sets their ownership to the specified account.
	#[endpoint(mint)]
	#[payable("EGLD")]
	fn mint(&self,
			count:u64,
			#[payment] payment: BigUint,
			new_token_collection: &Vec<u8>,
			new_token_description: &Vec<u8>,
			required_token:u64,
			secret: &Vec<u8>,
			initial_price: u32,
			max_markup:u16,
			properties:u16,
			owner:ManagedAddress,
			miner:ManagedAddress,
			creator:ManagedAddress,
			miner_ratio:u16,
			gift:u16,
			deadline:u64,
			limit:u8,
			money:TokenIdentifier
	) -> SCResult<Vec<u64>> {

		let mut caller=miner.clone();
		if !miner.is_zero() {
			caller=self.blockchain().get_caller();
		}

		//require!(properties & ONE_WINNER==0 || (properties & ONE_WINNER>0 && gift>0),"E45: Le reglage de ONE_WINNER est incorrect");
		require!(new_token_description.len() > 0,"E02: Title & description can't be empty together");

		//La limite du miner_ratio est à 10000 car on multiplie par 100 pour autoriser un pourcentage à 2 décimal
		require!(miner_ratio<=10000,"E04: La part du mineur doit etre entre 0 et 100");

		//Creation de la monnaie
		//voir https://github.com/ElrondNetwork/elrond-wasm-rs/blob/ed98b2b02bf95b7457c372f51b485ab69e019b58/elrond-wasm/src/types/general/token_identifier.rs

		require!(money.is_egld() || money.is_valid_esdt_identifier(),"E65: Invalid money");
		require!(gift==0 || miner==self.blockchain().get_caller(),"E88: Le créateur doit être le mineur si le token contient des fonds");

		if money.is_egld() && (properties & ONE_WINNER==0) {
			require!(payment == BigUint::from(gift as u64)*count,"E05: Transfert de fonds ne correspond pas au NFT");
		}

		require!(required_token==0 || required_token<=self.get_total_minted(),"E88: Le token requis n'existe pas");

		//On vérifie que tous les tokens devant contenir la recompense l'ont bien
		// if properties & ONE_WINNER==0 && ref_token_id!=MAX_U64 {
		// 	let first_token = self.tokens_map().get(ref_token_id);
		// 	require!(first_token.gift == gift,"E88: Ce token n'est pas identique à celui passé en référence");
		// }

		let token_ids=self.perform_mint(count,
									   caller,
									   owner,
									   creator,
									   new_token_collection,
									   new_token_description,
									   required_token,
									   secret,
									   initial_price,
									   properties,
									   miner_ratio,
									   gift,
									   deadline,
									   limit,
									   &money,0u8);

		return Ok(token_ids); //On retourne le token_id
	}




	/// Approves an account to transfer the token on behalf of its owner.<br>
	/// Only the owner of the token may call this function.
	#[endpoint(approve)]
	fn approve(&self, token_id: u64, approved_address: ManagedAddress) -> SCResult<()> {
		let token=self.tokens_map().get(token_id as usize);
		require!(token_id <= self.get_total_minted(), "E06: Token does not exist!");
		require!(self.blockchain().get_caller() == self.get_addresses(token.owner) ,"E07: Only the token owner can approve!");

		self.set_approval(token_id, &approved_address);

		return Ok(());
	}



	/// Revokes approval for the token.<br>
	/// Only the owner of the token may call this function.
	#[endpoint(revoke)]
	fn revoke(&self, token_id: u64) -> SCResult<()> {
		require!(token_id <= self.get_total_minted(), "E08: Token does not exist!");

		let token=self.tokens_map().get(token_id as usize);
		require!(self.blockchain().get_caller() == self.get_addresses(token.owner),"E09: Only the token owner can revoke approval!");

		if !self.approval_is_empty(token_id) {
			//TODO: on considère les approuvés comme des distributeurs, on doit donc supprimer le distributeur
			self.perform_revoke_approval(token_id);
		}

		return Ok(());
	}





	/// Transfer ownership of the token to a new account.
	#[endpoint(transfer)]
	fn transfer(&self, token_id: u64, to: ManagedAddress) -> SCResult<()> {
		require!(token_id <= self.get_total_minted(), "E12: Token does not exist!");
		let mut token=self.tokens_map().get(token_id as usize);

		//Le mineur peut avoir limité la possibilité de transfert du token à sa création
		require!(token.properties & CAN_TRANSFERT > 0,"E13: Ce token ne peut être offert");

		let caller = self.blockchain().get_caller();

		if caller == self.get_addresses(token.owner) {
			token.owner=self.set_addresses(&to);
			self.tokens_map().set(token_id as usize,&token);
			//self.perform_transfer(token_id, &self.get_addresses(&token,OWNER), &to);
			return Ok(());
		} else if !self.approval_is_empty(token_id) {
			//TODO code à conformer à ENFT
			let approved_address = self.get_approval(token_id);

			if caller == approved_address {
				token.owner=self.set_addresses(&to);
				self.tokens_map().set(token_id as usize,&token);

				//self.perform_transfer(token_id, &self.get_addresses(&token,OWNER), &to);
				return Ok(());
			}
		}
		return Ok(());
		//sc_error!("E14: Only the owner or the approved account may transfer the token!")
	}





	// Méthode privée utilisé pour effectivement créer le token
	//count permet de miner plusieurs tokens identique avec un seul appels
	fn perform_mint(&self,
					count:u64,
					new_token_miner: ManagedAddress,
					new_token_owner: ManagedAddress,
					new_token_creator: ManagedAddress,
					new_token_collection: &Vec<u8>,
					new_token_description: &Vec<u8>,
					required_token: u64,
					secret: &Vec<u8>,
					new_token_price: u32,
					properties:u16,
					miner_ratio:u16,
					gift:u16,
					deadline:u64,
					limit:u8,
					money: &TokenIdentifier,
					status:u8) -> Vec<u64> {

		let mut rc:Vec<u64>=Vec::new();

		let idx_token_owner=self.set_addresses(&new_token_owner);
		let idx_token_miner=self.set_addresses(&new_token_miner);
		let idx_token_creator=self.set_addresses(&new_token_creator);
		let idx_description=self.set_str(&mut new_token_description.to_vec());
		let idx_collection=self.set_str(&mut new_token_collection.to_vec());

		let mut temp_secret=secret.to_vec();

		//TODO Ajouter la Selection d'un billet gagnant pour le fonctionnement loterie
		let mut set_gift=gift;

		let mut required_tokens:Vec<u64>=Vec::new();
		if required_token>0 {
			required_tokens.push(required_token);  //TODO non satisfaisant car on ne peut jamais avoir le token 0 en référence
		}
		let offset=self.tokens_map().len() as u64;

		for id in offset..(count+offset) {

			//Substitution de chaines
			if temp_secret.eq_ignore_ascii_case(&Vec::from("@id@")) {
				temp_secret=id.to_be_bytes().to_ascii_uppercase();
			}
			let idx_secret=self.set_str(&mut temp_secret.to_vec());

			if properties & ONE_WINNER>0 {
				if gift>0 {
					if id % count == 0 {
						set_gift = gift;
						temp_secret= Vec::from("Gagné");
					} else {
						set_gift = 0;
						temp_secret= Vec::from("Perdu");
					}
				}
			}

			let token = Token {
				owner:idx_token_owner,
				miner:idx_token_miner,
				creator:idx_token_creator,
				price:new_token_price.clone(),
				resp:0u8,
				gift:set_gift,
				description:idx_description,
				secret:idx_secret,
				collection: idx_collection,
				status: status,
				deadline: deadline+self.blockchain().get_block_timestamp(),
				properties:properties,
				limit:limit,
				miner_ratio:miner_ratio,
				money: self.set_esdt(money),
				required:required_tokens.to_vec(),
			};

			self.tokens_map().push(&token);
			rc.push(id+1);
		}

		return rc;
	}



	fn perform_revoke_approval(&self, token_id: u64) {
		self.clear_approval(token_id);
	}



	fn perform_burn(self,token_id: u64,token: &mut Token) -> bool {
		if token.gift>0 {
			let miner_addr=self.get_addresses(token.miner);
			//Remboursement du créateur
			self.send_money(token.money,&miner_addr,BigUint::from(token.gift as u64),b"Miner refund");
		}

		token.miner=ZERO_ADDR;
		token.owner=ZERO_ADDR;
		self.tokens_map().set(token_id as usize,&token);

		return true;
	}


	//Détruit un token en lui affectant l'adresse 0x0 comme propriétaire et mineur
	#[endpoint(burn)]
	fn burn(&self, token_ids: Vec<u64>) -> SCResult<()> {
		let caller = self.blockchain().get_caller();

		for token_id in token_ids {
			require!(token_id<= self.get_total_minted(), "E15: Token does not exist!");

			let mut token = self.tokens_map().get(token_id as usize);

			require!(caller == self.get_addresses(token.owner) || (caller == self.get_addresses(token.miner) && (token.properties & MINER_CAN_BURN>0)),"E16: Only the owner account can burn token!");

			self.perform_burn(token_id, &mut token);
		}

		return Ok(());
	}



	//voir https://docs.elrond.com/developers/developer-reference/elrond-wasm-api-functions/#send-api
	fn send_money(&self,identifier:u16,dest:&ManagedAddress, amount:BigUint, comment:&[u8]) {
		let money=self.esdt_map().get(identifier as usize);
		self.send().direct(dest, &money, 0,&(amount), comment);
	}



	//Mise a jour du token
	#[endpoint(update)]
	fn update(&self, token_id: u64, field_name: &Vec<u8>,new_value: &Vec<u8>) -> SCResult<()>  {
		require!(token_id <= self.get_total_minted(), "Token does not exist!");
		let mut token=self.tokens_map().get(token_id as usize);

		let caller = self.set_addresses(&self.blockchain().get_caller());
		require!(caller == token.owner,"E10: Seul le propriétaire peut mettre a jour le NFT");
		require!(token.properties & FOR_SALE==0,"E52: Le NFT ne doit pas être en vente pour être modifié");
		require!(caller == token.miner,"Seul le créateur peut mettre a jour le token");

		if field_name.eq_ignore_ascii_case(&Vec::from("description")) {
			token.description= self.set_str(&new_value.to_vec());
		}

		self.tokens_map().set(token_id as usize,&token);

		return Ok(());
	}




	//Principe du vote
	#[endpoint(answer)]
	fn answer(&self, token_id: u64, response: u8) -> SCResult<()> {

		require!(token_id <= self.get_total_minted(), "E12: Token does not exist!");
		let mut token=self.tokens_map().get(token_id as usize);

		require!(!token.is_burn(self.blockchain().get_block_timestamp()),"E68: Ce token est détruit");

		let owner_addr=self.get_addresses(token.owner);
		let caller = self.blockchain().get_caller();
		require!(caller == owner_addr,"E10: Seul le propriétaire du token peut repondre");

		if token.gift>0 {
			//On récompense le participant
			self.send_money(token.money,&owner_addr,BigUint::from(token.gift as u64),b"pay for gift");
			token.gift=0;
		}

		token.resp=response;
		self.tokens_map().set(token_id as usize,&token);

		return Ok(());
	}



	//Retourne le contenu de la propriété secret du token en échange d'une vérification
	//que l'appelant est bien propriétaire du token
	//Si Response est non vide et Gift positif alors si response == secret on transfert le gift
	#[endpoint(open)]
	fn open(&self, token_id: u64, response: &Vec<u8>) -> SCResult<Vec<u8>> {
		require!(token_id <= self.get_total_minted(), "E12: Token does not exist!");
		let mut token=self.tokens_map().get(token_id as usize);

		require!(!token.is_burn(self.blockchain().get_block_timestamp()),"E23: Ce NFT n'existe plus");

		let caller = self.blockchain().get_caller();
		require!(caller == self.get_addresses(token.owner),"E10: Seul le propriétaire peut ouvrir le token");

		let mut secret=self.get_str(token.secret);

		require!(secret.len()>0 || token.gift>0,"E11: Ce token ne contient pas de secret");


		//let secret=mc.decrypt_base64_to_string(&token.secret).unwrap();
		//TODO: mettre en place le décryptage du secret
		//secret=self.decrypt(&secret);
		let eq=secret.eq(response);

		//https://docs.rs/openssl/0.10.32/openssl/rsa/index.html
		//let secret=v3::decrypt("secret",&enc_data);

		if token.gift>0 {
			//Si on est pas obligé de trouver le secret ou si la réponse est égale au secret on distribue les gains
			if token.properties & FIND_SECRET==0 || eq {
				self.send_money(token.money,&self.get_addresses(token.owner),BigUint::from(token.gift),b"pay for gift");
				token.gift=0;
				self.tokens_map().set(token_id as usize,&token);
				if secret.len()==0 {secret=Vec::from("Gagné");}
			}
		} else {
			if secret.len()==0 {
				secret=Vec::from("Perdu");
			}
		}


		//s'il fallait trouvé le secret
		if token.properties & FIND_SECRET>0 {
			if eq {
				secret=Vec::from("Gagné");
			} else {
				secret=Vec::from("Perdu");
			}
		}

		//Le token doit être auto-détruit
		if token.properties & SELF_DESTRUCTION>0 {
			self.perform_burn(token_id,&mut token);
		} else {
			//Le token doit retourner a son createur
			if token.properties & RENT>0 {
				self.transfer(token_id,self.get_addresses(token.miner));
			}
		}

		return Ok(secret);
	}


	//Permet d'ajouter un distributeur à la liste des distributeurs du token
	//Cette fonction est réservé au propriétaire du token
	#[endpoint(add_dealer)]
	fn add_dealer(&self,  token_ids: Vec<u64>, addr: &ManagedAddress, markup:u16) -> SCResult<()> {
		let owner_idx_addr=self.set_addresses(&self.blockchain().get_caller());

		let dealer_idx = self.get_idx_dealer(self.get_idx_addresses(addr));
		require!(dealer_idx!=NOT_FIND ,"E33: Distributeur inconnu");

		let mut dealer:Dealer=self.dealers_map().get(dealer_idx);

		for token_id in token_ids.clone() {
			let token = self.tokens_map().get(token_id as usize);
			require!(token.owner == owner_idx_addr,"E21: Only token owner can add dealer");
		}

		dealer.set_markup(token_ids.clone(),markup);


		//self.dealers_map().set(dealer_idx,&dealer);

		return Ok(());

		//sc_error!("le miner du token n'est pas autorisé par le dealer")
	}


	//Permet la mise en vente ou le retrait de la vente d'un token
	//Seul le propriétaire du token peut le remettre en vente
	//tag set_state
	#[endpoint(setstate)]
	fn setstate(&self,  token_ids: Vec<u64>,new_state:u8) -> SCResult<(u64)> {

		let mut rc=0;
		let caller_addr=self.set_addresses(&self.blockchain().get_caller());

		for token_id in token_ids {

			require!(token_id <= self.get_total_minted(), "E12: Token does not exist!");
			let mut token = self.tokens_map().get(token_id as usize);

			require!(caller_addr == token.owner ,"E17: Only token owner change state");
			require!(new_state == 0 || token.properties & CAN_RESELL > 0,"E18: Ce NFT ne peut être mise en vente");

			let old_state=token.properties;

			if new_state == 1 {
				token.properties=token.properties | FOR_SALE;
			} else {
				token.properties=token.properties & !FOR_SALE;
			}

			if token.properties != old_state {
				self.tokens_map().set(token_id as usize,&token);
				rc=rc+1;
			}

		}

		return Ok((rc));
	}




	//Recherche un dealer par son adresse
	//retourne dealer_count si on a pas trouvé le dealer
	#[view(find_dealer_by_addr)]
	fn find_dealer_by_addr(&self,dealer_addr: &ManagedAddress) -> Vec<u8> {
		let idx_dealer_addr=self.get_idx_addresses(dealer_addr);
		let dealer_idx=self.get_idx_dealer(idx_dealer_addr);
		let dealer=self.dealers_map().get(dealer_idx as usize);
		return self.dealer_to_vec(&dealer);
	}





	//Ajouter un miner approuvé à un dealer
	#[endpoint(add_miner)]
	fn add_miner(&self,  miner_addr: &ManagedAddress) -> SCResult<()> {
		let idx_dealer=self.get_idx_dealer(self.set_addresses(&self.blockchain().get_caller()));
		require!(idx_dealer != NOT_FIND, "E55:Dealer not listed");

		let mut dealer=self.dealers_map().get(idx_dealer);
		dealer.miners.push(self.set_addresses(&miner_addr));
		self.dealers_map().set(idx_dealer,&dealer);

		return Ok(());
	}


	//Supprimer un miner approuvé à un dealer
	#[endpoint(del_miner)]
	fn del_miner(&self,  miner_addr: &ManagedAddress) -> SCResult<()> {
		let idx_dealer=self.get_idx_dealer(self.get_idx_addresses(&self.blockchain().get_caller()));

		let idx_miner_addr=self.get_idx_addresses(miner_addr);

		let mut dealer=self.dealers_map().get(idx_dealer);
		dealer.miners.retain(|&addr| addr != idx_miner_addr);  //https://doc.rust-lang.org/std/vec/struct.Vec.html#method.retain
		self.dealers_map().set(idx_dealer,&dealer);

		return Ok(());
	}


	#[endpoint(dealer_state)]
	fn dealer_state(&self,  state: u8) -> SCResult<()> {
		let addr = self.get_idx_addresses(&self.blockchain().get_caller());

		let idx=self.get_idx_dealer(addr);
		require!(idx != NOT_FIND,"Dealer not listed");
		let mut dealer=self.dealers_map().get(idx);

		dealer.state=state;
		self.dealers_map().set(idx,&dealer);
		return Ok(());
	}


	//Ajout un nouveau distributeur
	//state=0 open / 1 close
	#[endpoint(new_dealer)]
	fn new_dealer(&self) -> SCResult<u64> {
		let addr=self.set_addresses(&self.blockchain().get_caller());
		let dealer_idx=self.get_idx_dealer(addr);
		if dealer_idx == NOT_FIND {
			let mut new_dealer = Dealer {
				state: 0,
				addr: addr,
				miners: Vec::new(),
				markups: Vec::new(),
				max_markups: Vec::new(),
				tokens:Vec::new()
			};
			new_dealer.miners.push(addr); //On ajoute le dealer comme créateur
			self.dealers_map().push(&new_dealer);
		}
		return Ok(self.dealers_map().len() as u64);
	}



	#[view(is_miner)]
	fn is_miner(&self,  miner_addr: ManagedAddress) -> bool {
		return self.ipfs_map().contains_key(&miner_addr);
	}



	//Retourne la liste des mineurs approuvé par un distributeurs (separateur de liste : 000000)
	#[view(miners)]
	fn miners(&self,  dealer_addr: ManagedAddress) -> Vec<u8> {

		let mut rc=Vec::new();
		let dealer_idx=self.get_idx_dealer(self.get_idx_addresses(&dealer_addr));
		if dealer_idx == NOT_FIND {
			return rc;
		}

		let dealer=self.dealers_map().get(dealer_idx);

		for miner in dealer.miners.iter() {
			rc.append(&mut self.get_addresses(*miner).to_address().to_vec());
		}

		return rc;
	}



	#[view(votes)]
	fn get_votes(&self,filter_miner:ManagedAddress) -> Vec<u8> {

		let mut results=Vec::new();

		for _i in 0u8..10u8 {
			results.push(0u8);
		}

		for idx in 0..self.get_total_minted() {
			let token=self.tokens_map().get(idx as usize);
			if self.get_addresses(token.miner)==filter_miner {
				results[token.resp as usize]=results[token.resp as usize]+1;
			}
		}
		return results;
	}


	//retourne l'ensemble des distributeurs référencés si l'adresse est 0x0 ou les distributeurs d'un mineur
	#[view(dealers)]
	fn get_dealers(&self,filter_miner:&ManagedAddress) -> Vec<u8> {
		let mut rc=Vec::new();
		let idx_filter_miner=self.get_idx_addresses(filter_miner);

		for dealer in self.dealers_map().load_as_vec() {
			if idx_filter_miner == ZERO_ADDR || dealer.miners.contains(&idx_filter_miner) {
				rc.append(&mut self.dealer_to_vec(&dealer))
			}
		}
		return rc;
	}


	#[view(all_dealers)]
	fn get_all_dealers(&self) -> Vec<u8> {
		return self.get_dealers(&ManagedAddress::zero())
	}


	#[view(get_dealer_by_idx)]
	fn get_dealer_by_idx(&self,id: u64) -> Vec<u8> {
		return self.dealer_to_vec(&self.dealers_map().get(id as usize));
	}




		//efface l'ensemble des distributeurs
	#[endpoint]
	fn clear_dealer(&self,  token_id: u64) -> SCResult<()> {
		let token = self.tokens_map().get(token_id as usize);

		let caller=self.blockchain().get_caller();

		require!(token_id <= self.get_total_minted(), "E12: Token does not exist!");
		require!(self.get_addresses(token.owner) == caller,"E23: Only token owner can remove dealer");

		self.tokens_map().set(token_id as usize,&token);

		return Ok(())
	}




	//Modifier le prix (dans la fourchette initialement défini par le mineur du token)
	//Seul les distributeurs peuvent modifier le prix
	#[endpoint]
	fn price(&self, token_ids: Vec<u64>, markup: u16) -> SCResult<()> {

		let dealer_idx = self.get_idx_dealer(self.get_idx_addresses(&self.blockchain().get_caller()));
		let mut dealer:Dealer=self.dealers_map().get(dealer_idx);

		for token_id in token_ids.clone() {
			let token=self.tokens_map().get(token_id as usize);
			let (markup,max_markup)=dealer.find_markup(token_id);
			require!(markup <= max_markup,"E89: Interval de modification du prix dépassé");
		}

		dealer.set_markup(token_ids.clone(),markup);

		return Ok(());
	}


	#[view(count_by_collection)]
	fn count_by_collection(&self,owner_filter:u64,creator_filter:u64,idx_collection_filter:u64) -> u8 {
		let mut rc=0u8;
		let now=self.blockchain().get_block_timestamp();
		for token in self.tokens_map().load_as_vec() {
			if token.owner==owner_filter &&
				(idx_collection_filter==token.collection || idx_collection_filter==0) &&
				(creator_filter==token.creator || creator_filter==ZERO_ADDR) &&
				!token.is_burn(now)
			{
				rc=rc+1;
			}
		}
		return rc;
	}

	//Fonction d'achat d'un token
	//token_id: désigne le token à acheter
	//dealer: déclare le vendeur qui à fait la vente. Cela permet au système de récupéré le prix avec la commission et de procéder au reversement
	//Voir l'exemple de la fonction fund dans https://github.com/ElrondNetwork/elrond-wasm-rs/blob/master/contracts/examples/crowdfunding-esdt/src/crowdfunding_esdt.rs
	#[payable("EGLD")]
	#[endpoint]
	fn buy(&self, token_id: u64,dealer_addr:ManagedAddress) -> SCResult<(u64)> {
		let (payment, _pay_token)=self.call_value().payment_token_pair();

		let idx_dealer=self.get_idx_dealer(self.get_idx_addresses(&dealer_addr));
		require!(idx_dealer != NOT_FIND,"E55: Distributeur inconnu");

		require!(token_id <= self.get_total_minted(), "E12: Token does not exist!");
		let mut token = self.tokens_map().get(token_id as usize);

		let idx_caller=self.set_addresses(&self.blockchain().get_caller());
		require!(token.owner != idx_caller,"E29: Ce token vous appartient déjà");
		require!(token.properties & FOR_SALE>0,"E30: Ce token n'est pas en vente");

		if token.required.len() > 0 {
			let mut bc=true;
			for idx_token_required in token.required.iter() {
				let token_required=self.tokens_map().get(*idx_token_required as usize);
				if !self.owner_has_required_token(idx_caller,token_required.description,token_required.miner) {
					bc=false;
				}
			}
			require!(bc,"E66: Vous ne posséder pas les tokens requis")
		}

		if token.limit>0 {
			require!(self.count_by_collection(idx_caller,token.creator,token.collection)<token.limit,"E67: Vous avez dépassé la limite de NFT de cette collection");
		}

		//On retrouve le distributeur du token
		let dealer:Dealer=self.dealers_map().get(idx_dealer);
		let (mark_up,max_markup)=dealer.find_markup(token_id);
		let mut payment_for_dealer=CONVERT_TO_GAS*(mark_up as u64);

		require!(token.properties & DIRECT_SELL>0 || !dealer.is_zero() ,"E31: La vente directe n'est pas autorisé");


		//calcul du payment au owner
		let mut payment_for_owner=payment-BigUint::from(payment_for_dealer);
		//Dans le cas d'un ESDT on corrige la valeur de payment en attendant de savoir comment la passer en argument depuis python
		let money=self.esdt_map().get(token.money as usize);
		if money.is_esdt() {
			payment_for_owner=BigUint::from(CONVERT_TO_GAS*token.price as u64)-BigUint::from(payment_for_dealer);
		} else {
			require!(payment_for_owner >= BigUint::from(token.price.clone() as u64),"E33: Paiement non égale à celui du token");
		}

		if !dealer.is_zero() && payment_for_dealer>0 {
			//On retribue le mineur sur la commission du distributeur
			if token.miner_ratio>0 {
				let (markup,max_markup)=dealer.find_markup(token_id);
				let payment_for_miner=1000000000000*(markup as u64)*token.miner_ratio as u64;
				self.send_money(token.money,&self.get_addresses(token.miner),BigUint::from(payment_for_miner),b"miner pay");
				payment_for_dealer=payment_for_dealer-payment_for_miner;
			}

			//Transaction issue d'un revendeur
			self.send_money(token.money,&self.get_addresses(dealer.addr),BigUint::from(payment_for_dealer),b"dealer pay");
		}

		if payment_for_owner > BigUint::from(0u64) {
			self.send_money(token.money,&self.get_addresses(token.owner),payment_for_owner,b"owner pay");
		}

		token.properties=&token.properties & !FOR_SALE;//Le token n'est plus a vendre
		token.owner=idx_caller; //On change le propriétaire
		self.tokens_map().set(token_id as usize,&token);

		return Ok((token_id));
	}



	// fn get_dealer_addresses_for_token(&self,id_token:u64 ) -> Vec<u64> {
	// 	let mut rc=Vec::new();
	// 	for dealer in self.dealers_map().load_as_vec(): Vec<Dealer> {
	// 		if dealer.tokens.contains(&id_token){
	// 			rc.push(dealer.addr);
	// 			break;
	// 		}
	// 	}
	// 	return rc;
	// }



	fn vec_equal(&self,va: &Vec<u8>, vb: &Vec<u8>) -> bool {
		if va.len()!= vb.len() {return false};
		for i in 0..va.len() {
			if va[i]!=vb[i] {return false;}
		}
		return true;
	}




	fn _is_in(&self,token: &Token , _list_tokens:&Vec<Token>) -> bool {
		if token.properties & UNIK==0 {
			return false;
		}
		// for t in list_tokens {
		// 	if t.miner==token.miner {
		// 		let mut comp=t;
		// 		if token.title[0]>0 {
		// 			comp=self.complete_token(t);
		// 		}
		// 		if comp.title==token.title {
		// 			return true;
		// 		}
		// 	}
		// }
		return false;
	}


	fn token_to_vec(&self,token:Token,markup:u16,max_markup:u16,token_id:u64) -> Vec<u8> {
		//Chargement des contenus
		let collection=self.get_str(token.collection);
		let len_collection=collection.len() as u16;

		let description=self.get_str(token.description);
		let len_description=description.len() as u16;

		let token_owner_addr=self.get_addresses(token.owner);
		let token_miner_addr=self.get_addresses(token.miner);
		let token_creator_addr=self.get_addresses(token.creator);

		let mut item:Vec<u8>=Vec::new();

		//On commence par inscrire la taille de token_price & title dont les tailles dépendent du contenu
		//doc sur le conversion :https://docs.rs/elrond-wasm/0.10.3/elrond_wasm/
		item.append(&mut len_collection.to_be_bytes().to_vec());
		item.append(&mut len_description.to_be_bytes().to_vec());
		item.append(&mut self.esdt_map().get(token.money as usize).as_name().len().to_be_bytes().to_vec());

		//Puis on ajoute l'ensemble des informations d'un token
		//dans un vecteur d'octets
		let price=token.price+100*markup as u32;

		let mut has_secret=0u8;
		if token.secret>0 || token.gift>0 {
			has_secret=1u8;
		}

		item.append(&mut price.to_be_bytes().to_vec());
		item.append(&mut self.esdt_map().get(token.money as usize).as_name().into_vec());
		item.append(&mut token_owner_addr.to_address().to_vec());
		item.push(has_secret);

		item.push(token.properties.to_be_bytes()[0]);
		item.push(token.properties.to_be_bytes()[1]);
		item.push(token.status);

		if token.properties & SECRET_VOTE>0 {
			item.push(0u8);
		} else {
			item.push(token.resp);
		}

		item.append(&mut max_markup.to_be_bytes().to_vec());
		item.append(&mut markup.to_be_bytes().to_vec());
		item.append(&mut token.deadline.to_be_bytes().to_vec());
		item.append(&mut token.miner_ratio.to_be_bytes().to_vec());
		item.append(&mut token_miner_addr.to_address().to_vec());
		item.append(&mut token_creator_addr.to_address().to_vec());
		item.append(&mut token_id.to_be_bytes().to_vec()); //Identifiant du token
		//item.append(&mut token.collection.to_be_bytes().to_vec());
		item.append(&mut collection.to_vec());
		item.append(&mut description.to_vec());

		return item;
	}




	//Tag /nfts get_nfts tokens /tokens
	//Récupérer l'ensemble des tokens en appliquant les filtres sauf si celui est à la valeur 0x0
	//seller: uniquement les tokens dont "seller" fait parti des distributeurs déclarés
	//owner: uniquement les tokens dont le propriétaire est "owner"
	//miner: uniquement les tokens fabriqués par "miner"
	#[view(tokens)]
	fn tokens(&self,dealer_filter: &ManagedAddress,owner_filter: &ManagedAddress, miner_filter: &ManagedAddress, limit:u64) -> Vec<Vec<u8>> {
		let mut rc:Vec<Vec<u8>>=Vec::new();

		//voir https://docs.elrond.com/developers/developer-reference/elrond-wasm-api-functions/
		let idx_miner_filter=self.set_addresses(miner_filter);
		let idx_owner_filter=self.set_addresses(owner_filter);
		let idx_dealer_filter=self.set_addresses(dealer_filter);

		let mut occ=1;
		let now=self.blockchain().get_block_timestamp();

		if idx_dealer_filter==ZERO_ADDR {
			//Ici on retourne tous les tokens
			for i in 2..self.get_total_minted()+1 {
				if limit<=occ {break;}
				let token:Token=self.tokens_map().get(i as usize);
				if (idx_owner_filter == ZERO_ADDR || idx_owner_filter == token.owner) && (idx_miner_filter == ZERO_ADDR || idx_miner_filter == token.miner) {
					if !token.is_burn(now) {
						rc.push(self.token_to_vec(token,0u16,0u16,i));
						occ=occ+1;
					}
				}
			}
		} else {
			let idx_dealer=self.get_idx_dealer(idx_dealer_filter);
			let dealer:Dealer = self.dealers_map().get(idx_dealer);
			for token_id in dealer.tokens.clone() {
				let token:Token=self.tokens_map().get(token_id as usize);
				let (markup,max_markup)=dealer.find_markup(token_id);
				if !token.is_burn(now) {
					rc.push(self.token_to_vec(token,markup,max_markup,token_id));
				}
			}
		}
		return rc;
	}



	//Fonctions utilisées pour les NFT
	// #[view(tokenOwner)]
	// #[storage_get("tokenOwner")]
	// fn get_token_owner(&self, token_id: u64) -> ManagedAddress;
	// #[storage_set("tokenOwner")]
	// fn set_token_owner(&self, token_id: u64, owner: &ManagedAddress);



	fn get_total_minted(&self) -> u64 {
		return self.tokens_map().len() as u64;
	}

	#[warn(deprecated)]
	#[storage_mapper("ipfs")]
	fn ipfs_map(&self) -> MapMapper<ManagedAddress, BigUint>;

	//Voir l'aide https://docs.elrond.com/developers/developer-reference/smart-contract-developer-best-practices/ section Storage mapper
	//voir également des exemples dans https://github.com/ElrondNetwork/elrond-wasm-rs/blob/c794a7f6e7b54054ca6efd708634f9b549644610/contracts/examples/multisig/src/multisig_state.rs
	#[view(addresses)]
	#[storage_mapper("addresses")]
	fn addresses(&self) -> VecMapper<ManagedAddress>;

	#[view(ESDT_map)]
	#[storage_mapper("ESDT_map")]
	fn esdt_map(&self) -> VecMapper<TokenIdentifier>;


	// Récupération d'un dealer
	#[view(dealers_map)]
	#[storage_mapper("dealers_map")]
	fn dealers_map(&self) -> VecMapper<Dealer>;

	#[view(strs)]
	#[storage_mapper("strs")]
	fn strs(&self) -> VecMapper<Vec<u8>>;


	//Information sur les mineurs / créateurs
	// #[view(getMinerInfos)]
	// #[storage_get("minerInfos")]
	// fn get_miner_infos(&self,  miner: &ManagedAddress) -> Vec<u8>;
	// #[storage_set("minerInfos")]
	// fn set_miner_infos(&self, miner: &ManagedAddress, infos: Vec<u8>);



	// Fonction d'approbation pour maintient de compatibilité avec les NFT
	#[storage_is_empty("approval")]
	fn approval_is_empty(&self, token_id: u64) -> bool;
	#[view(approval)]
	#[storage_get("approval")]
	fn get_approval(&self, token_id: u64) -> ManagedAddress;
	#[storage_set("approval")]
	fn set_approval(&self, token_id: u64, approved_address: &ManagedAddress);
	#[storage_clear("approval")]
	fn clear_approval(&self, token_id: u64);


}

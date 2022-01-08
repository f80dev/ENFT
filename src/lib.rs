//GNU GENERAL PUBLIC LICENSE - Version 3, 29 June 2007
//Auteur: Herve Hoareau

#![no_std]

elrond_wasm::imports!();
use token::Token;
use dealer::Dealer;
mod token;
mod dealer;
use elrond_wasm::elrond_codec::TopDecodeInput;

const CONVERT_TO_GAS:u64 	=10000000000000000; //Facteur de conversion en gas

const IS_MASTER:u8    	    =0b00000010; //est utilisé par un clone (dans ce cas là le token ne peut être brulé)
const IS_CLONE:u8    	    =0b00000001; //Le token ne peut être possédé qu'une seul fois

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

const MINER:u8 = 0;
const OWNER:u8 = 1;
const NOT_FIND:usize = 65535;
const MAX_U64:u64=4294967296;


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
	fn set_addresses(&self,new_addr: &ManagedAddress) -> u32 {
		let mut rc=self.addresses();
		let mut idx =rc.load_as_vec().iter().position(|r| r.eq(new_addr)).unwrap_or(NOT_FIND);
		if idx == NOT_FIND {
			idx = rc.len();
			rc.push(&new_addr);
		}
		return (idx) as u32; //Car la lecture du vecteur commence à 1
	}





	fn set_str(&self,vec: &Vec<u8>) -> u64 {
		let mut rc=self.strs();
		let mut idx =rc.load_as_vec().iter().position(|r| r == vec).unwrap_or(NOT_FIND);
		if idx == NOT_FIND {
			idx = rc.len();
			rc.push(vec);
		}
		return (idx) as u64; //Car la lecture du vecteur commence à 1
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



	//Retourne une adresse de token du référentiel d'adresses
	fn get_esdt(&self,token: &Token) -> TokenIdentifier {
		return self.esdt_map().get(token.money as usize);
	}



	#[init]
	fn init(&self,initial_value: u64) {
		let owner = self.blockchain().get_caller();
		self.set_owner(&owner);
		self.set_str(&Vec::from(""));
		self.set_addresses(&ManagedAddress::zero()); //La première address de la liste est l'adresse 0
	}


	// fn decrypt(&self,secret: &Vec<u8>) -> Vec<u8> {
	// 	//TODO implémenter ici un fonction de décryptage
	// 	return secret;
	// }

	#[endpoint]
	#[payable("EGLD")]
	fn clone(&self,#[payment] _payment: BigUint,ref_token_id:u64,count: u64,owner:ManagedAddress) -> SCResult<u64> {
		let mut token=self.complete_token(ref_token_id);

		let miner=self.addresses().get(token.miner as usize);
		require!(miner==self.blockchain().get_caller(),"E88: Seul le miner peut cloner un token");

		let money=self.get_esdt(&token);

		let token_secret=self.strs().get(token.secret as usize);
		let token_description=self.strs().get(token.description as usize);
		let token_collection=self.strs().get(token.collection as usize);

		let token_id=self.perform_mint(count,
									   miner,
									   owner,
									   &token_collection,
									   &token_description,
									   &token_secret,
									   token.price,
									   token.min_markup,
									   token.max_markup,
									   token.properties,
									   token.miner_ratio,
									   token.gift,
									   &money,
									   0u8);

		//On bascule le statut du token master
		token.status=token.status | IS_MASTER;
		self.tokens_map().set(ref_token_id as usize,&token);

		return Ok(ref_token_id) //On retourne le ref_token_id
	}


	/// Creates new tokens and sets their ownership to the specified account.
	/// Only the contract owner may call this function.
	#[endpoint]
	#[payable("EGLD")]
	fn mint(&self,
			count:u64,
			#[payment] payment: BigUint,
			new_token_collection: &Vec<u8>,
			new_token_description: &Vec<u8>,
			secret: &Vec<u8>,
			initial_price: u32,
			min_markup:u16,
			max_markup:u16,
			properties:u16,
			owner:ManagedAddress,
			miner:ManagedAddress,
			miner_ratio:u16,
			gift:u16,
			money:TokenIdentifier
	) -> SCResult<u64> {

		let mut caller=miner.clone();
		if !miner.is_zero() {caller=self.blockchain().get_caller();}

		//require!(properties & ONE_WINNER==0 || (properties & ONE_WINNER>0 && gift>0),"E45: Le reglage de ONE_WINNER est incorrect");
		require!(new_token_description.len() > 0,"E02: Title & description can't be empty together");
		require!(min_markup <= max_markup,"E03: L'interval de commission est incorrect");

		//La limite du miner_ratio est à 10000 car on multiplie par 100 pour autoriser un pourcentage à 2 décimal
		require!(miner_ratio<=10000,"E04: La part du mineur doit etre entre 0 et 100");

		//Creation de la monnaie
		//voir https://github.com/ElrondNetwork/elrond-wasm-rs/blob/ed98b2b02bf95b7457c372f51b485ab69e019b58/elrond-wasm/src/types/general/token_identifier.rs

		require!(money.is_egld() || money.is_valid_esdt_identifier(),"E65: Invalid money");

		if money.is_egld() && (properties & ONE_WINNER==0) {
			require!(payment>=BigUint::from(gift as u64),"E05: Transfert de fonds insuffisant pour le token de reference");
		}

		//On vérifie que tous les tokens devant contenir la recompense l'ont bien
		// if properties & ONE_WINNER==0 && ref_token_id!=MAX_U64 {
		// 	let first_token = self.tokens_map().get(ref_token_id);
		// 	require!(first_token.gift == gift,"E88: Ce token n'est pas identique à celui passé en référence");
		// }

		let token_id=self.perform_mint(count,
									   miner,
									   owner,
									   new_token_collection,
									   new_token_description,
									   secret,
									   initial_price,
									   min_markup,
									   max_markup,
									   properties,
									   miner_ratio,
									   gift,
									   &money,0u8);
		return Ok(token_id) //On retourne le token_id
	}




	/// Approves an account to transfer the token on behalf of its owner.<br>
	/// Only the owner of the token may call this function.
	// #[endpoint]
	fn approve(&self, token_id: u64, approved_address: ManagedAddress) -> SCResult<()> {
		let token=self.tokens_map().get(token_id as usize);
		require!(token_id < self.get_total_minted(), "E06: Token does not exist!");
		require!(self.blockchain().get_caller() == self.addresses().get(token.owner as usize) ,"E07: Only the token owner can approve!");

		self.set_approval(token_id, &approved_address);

		Ok(())
	}



	/// Revokes approval for the token.<br>
	/// Only the owner of the token may call this function.
	// #[endpoint]
	fn revoke(&self, token_id: u64) -> SCResult<()> {
		require!(token_id < self.get_total_minted(), "E08: Token does not exist!");

		let token=self.tokens_map().get(token_id as usize);
		require!(self.blockchain().get_caller() == self.addresses().get(token.owner as usize),"E09: Only the token owner can revoke approval!");

		if !self.approval_is_empty(token_id) {
			//TODO: on considère les approuvés comme des distributeurs, on doit donc supprimer le distributeur
			self.perform_revoke_approval(token_id);
		}

		Ok(())
	}





	/// Transfer ownership of the token to a new account.
	#[endpoint]
	fn transfer(&self, token_id: u64, to: ManagedAddress) -> SCResult<()> {
		require!(token_id < self.get_total_minted(), "E12: Token does not exist!");
		let mut token=self.tokens_map().get(token_id as usize);

		//Le mineur peut avoir limité la possibilité de transfert du token à sa création
		require!(token.properties & CAN_TRANSFERT > 0,"E13: Ce token ne peut être offert");

		let caller = self.blockchain().get_caller();

		if caller == self.addresses().get(token.owner as usize) {
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
		sc_error!("E14: Only the owner or the approved account may transfer the token!")
	}





	// Méthode privée utilisé pour effectivement créer le token
	//count permet de miner plusieurs tokens identique avec un seul appels
	fn perform_mint(&self,
					count:u64,
					new_token_miner: ManagedAddress,
					new_token_owner: ManagedAddress,
					new_token_collection: &Vec<u8>,
					new_token_description: &Vec<u8>,
					secret: &Vec<u8>,
					new_token_price: u32,
					min_markup: u16, max_markup: u16,
					properties:u16,
					miner_ratio:u16,
					gift:u16,
					money: &TokenIdentifier,
					status:u8) -> u64 {

		let total_minted = self.get_total_minted();
		let first_new_id = total_minted;
		let last_new_id = total_minted + count;

		let mut temp_secret=secret.to_vec();

		//Selection d'un billet gagnant pour le fonctionnement loterie
		let mut set_gift=gift;
		let id_money=self.set_esdt(money);

		for id in first_new_id..last_new_id {

			//Substitution de chaines
			if temp_secret.eq_ignore_ascii_case(&Vec::from("@id@")) {
				temp_secret=id.to_be_bytes().to_ascii_uppercase();
			}

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
				owner:self.set_addresses(&new_token_owner),
				miner:self.set_addresses(&new_token_miner),
				price:new_token_price.clone(),
				resp:0u8,
				gift:set_gift,
				description:self.set_str(&mut new_token_description.to_vec()),
				secret:self.set_str(&mut temp_secret.to_vec()),
				collection: self.set_str(&mut new_token_collection.to_vec()),
				status: status,
				min_markup:min_markup,
				max_markup:max_markup,
				dealers: 0u8,
				dealer_ids:Vec::new(),
				dealer_markup:Vec::new(),
				properties:properties,
				miner_ratio:miner_ratio,
				money: id_money
			};

			self.tokens_map().push(&token);
		}

		return first_new_id;
	}



	fn perform_revoke_approval(&self, token_id: u64) {
		self.clear_approval(token_id);
	}



	fn perform_burn(self,token_id: u64,token: &mut Token) -> bool {
		if token.gift>0 {
			let miner_addr=self.addresses().get(token.miner as usize);
			//Remboursement du créateur
			self.send_money(&token,&miner_addr,BigUint::from(token.gift as u64*10000000000000000),b"Miner refund");
		}

		token.miner=0;
		token.owner=0;
		self.tokens_map().set(token_id as usize,&token);

		return true;
	}


	//Détruit un token en lui affectant l'adresse 0x0 comme propriétaire et mineur
	#[endpoint]
	fn burn(&self, token_ids: Vec<u64>) -> SCResult<()> {
		let caller = self.blockchain().get_caller();
		let total_minted=self.get_total_minted();

		for token_id in token_ids {
			require!(token_id< total_minted, "E15: Token does not exist!");

			let mut token = self.tokens_map().get(token_id as usize);

			require!(caller == self.addresses().get(token.owner as usize) || (caller == self.addresses().get(token.miner as usize) && (token.properties & MINER_CAN_BURN>0)),"E16: Only the owner account can burn token!");

			self.perform_burn(token_id, &mut token);
		}

		return Ok(());
	}




	fn send_money(&self,token:&Token,dest:&ManagedAddress, amount:BigUint, comment:&[u8]) {
		let money=self.get_esdt(&token);
		if money.is_egld() {
			self.send().direct_egld(dest,&amount,comment);
		} else {
			self.send().direct(dest, &money, 0,&(amount), comment);
		}
	}



	//Mise a jour du token
	// #[endpoint]
	// fn update(&self, token_id: u64, field_name: &Vec<u8>,new_value: &Vec<u8>) -> SCResult<()>  {
	// 	require!(token_id < self.get_total_minted(), "Token does not exist!");
	// 	let mut token=self.tokens_map().get(token_id as usize);
	//
	// 	let caller = self.blockchain().get_caller();
	// 	require!(caller == self.get_addresses(&token,OWNER),"E10: Seul le propriétaire peut mettre a jour le NFT");
	// 	require!(token.properties & FOR_SALE==0,"E52: Le token ne doit pas être en vente");
	// 	require!(caller == self.get_addresses(&token,MINER),"Seul le créateur peut mettre a jour le token");
	//
	// 	if field_name.eq_ignore_ascii_case(&Vec::from("description")) {
	// 		token.description= self.set_str(&new_value.to_vec());
	// 	}
	//
	// 	self.tokens_map().set(token_id as usize,&token);
	//
	// 	return Ok(());
	// }




	//Principe du vote
	#[endpoint]
	fn answer(&self, token_id: u64, response: u8) -> SCResult<()> {

		require!(token_id < self.get_total_minted(), "Token does not exist!");
		let mut token=self.tokens_map().get(token_id as usize);

		let owner_addr=self.addresses().get(token.owner as usize);
		let caller = self.blockchain().get_caller();
		require!(caller == owner_addr,"E10: Seul le propriétaire du token peut repondre");

		if token.gift>0 {
			//On récompense le participant
			self.send_money(&token,&owner_addr,BigUint::from(CONVERT_TO_GAS * token.gift as u64),b"pay for gift");
			token.gift=0;
		}

		token.resp=response;
		self.tokens_map().set(token_id as usize,&token);

		return Ok(());
	}



	//Retourne le contenu de la propriété secret du token en échange d'une vérification
	//que l'appelant est bien propriétaire du token
	//Si Response est non vide et Gift positif alors si response == secret on transfert le gift
	#[endpoint]
	fn open(&self, token_id: u64, response: &Vec<u8>) -> SCResult<Vec<u8>> {
		require!(token_id < self.get_total_minted(), "Token does not exist!");
		let mut token=self.complete_token(token_id);

		let caller = self.blockchain().get_caller();
		require!(caller == self.addresses().get(token.owner as usize),"E10: Seul le propriétaire peut ouvrir le token");

		let mut secret=self.strs().get(token.secret as usize);

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
				self.send_money(&token,&self.addresses().get(token.owner as usize),BigUint::from(10000000000000000*token.gift as u64),b"pay for gift");
				token.gift=0;
				self.tokens_map().set(token_id as usize,&token);
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
				self.transfer(token_id,self.addresses().get(token.miner as usize));
			}
		}

		return Ok(secret);
	}




	//Permet la mise en vente ou le retrait de la vente d'un token
	//Seul le propriétaire du token peut le remettre en vente
	//tag set_state
	#[endpoint]
	fn setstate(&self,  token_ids: Vec<u64>,new_state:u8) -> SCResult<(u64)> {

		let mut rc=0;
		let caller=self.blockchain().get_caller();
		for token_id in token_ids {
			require!(token_id < self.get_total_minted(), "E19: Token does not exist!");

			let mut token = self.tokens_map().get(token_id as usize);

			require!(self.addresses().get(token.owner as usize) == caller ,"E17: Only token owner change state");
			require!(token.properties & CAN_RESELL>0,"E18: Ce NFT ne peut être mise en vente");

			let old_state=token.properties;
			if new_state==1 {
				token.properties=token.properties | FOR_SALE;
			} else {
				token.properties=token.properties & !FOR_SALE;
			}

			if token.properties!=old_state {
				self.tokens_map().set(token_id as usize,&token);
				rc=rc+1;
			}

		}

		return Ok((rc));
	}




	//Recherche un dealer par son adresse
	//retourne dealer_count si on a pas trouvé le dealer
	fn find_dealer_by_addr(&self,dealer_addr: &ManagedAddress) -> usize {
		let total=self.dealers_map().len();
		for i in 0..total {
			let dealer=self.dealers_map().get(i);
			if &dealer.addr==dealer_addr {
				return i;
			}
		}
		return total;
	}


	//Recherche un dealer par son adresse dans un token
	fn find_dealer_in_token(&self,idx_addr:u32 ,token:&Token) -> usize {
		let mut rc=NOT_FIND;
		if idx_addr!=0 {
			rc=token.dealer_ids.iter().position(|x| *x == idx_addr).unwrap_or(NOT_FIND);
		}
		return rc;
	}


	//Ajouter un miner approuvé à un dealer
	#[endpoint]
	fn add_miner(&self,  miner_addr: &ManagedAddress) -> SCResult<()> {
		let dealer_id=self.find_dealer_by_addr(&self.blockchain().get_caller());
		require!(dealer_id < self.dealers_map().len(), "Dealer not listed");

		let mut dealer=self.dealers_map().get(dealer_id as usize);

		dealer.miners.push(miner_addr.clone());
		self.dealers_map().set(dealer_id,&dealer);

		//self.ipfs_map().insert(miner_addr.clone(),ipfs_token);

		return Ok(());
	}


	//Supprimer un miner approuvé à un dealer
	#[endpoint]
	fn del_miner(&self,  miner_addr: &ManagedAddress) -> SCResult<()> {
		let dealer_id=self.find_dealer_by_addr(&self.blockchain().get_caller());
		require!(dealer_id < self.dealers_map().len() , "Dealer not listed");

		let mut dealer=self.dealers_map().get(dealer_id);

		let mut idx=0;
		for miner in dealer.miners.iter() {
			if miner == miner_addr {
				dealer.miners.remove(idx);
				self.dealers_map().set(dealer_id,&dealer);
				break;
			}
			idx=idx+1;
		}

		return Ok(());
	}


	#[endpoint]
	fn dealer_state(&self,  state: u8) -> SCResult<()> {
		let addr = self.blockchain().get_caller();
		let idx = self.find_dealer_by_addr(&addr);

		require!(idx<self.dealers_map().len(),"Dealer not listed");
		let mut dealer=self.dealers_map().get(idx);

		dealer.state=state;
		self.dealers_map().set(idx,&dealer);
		return Ok(());
	}


	//Ajout un nouveau distributeur
	//state=0 open / 1 close
	#[endpoint]
	fn new_dealer(&self) -> SCResult<usize> {
		let addr=self.blockchain().get_caller();
		let idx=self.find_dealer_by_addr(&addr);
		if idx == self.dealers_map().len() {
			let dealer = Dealer {
				state: 0,
				addr: addr.clone(),
				miners: Vec::new()
			};
			self.dealers_map().push(&dealer);
		}
		Ok(idx)
	}



	#[view(is_miner)]
	fn is_miner(&self,  miner_addr: ManagedAddress) -> bool {
		return self.ipfs_map().contains_key(&miner_addr);
	}



	//Retourne la liste des mineurs approuvé par un distributeurs (separateur de liste : 000000)
	#[view(miners)]
	fn miners(&self,  dealer_addr: ManagedAddress) -> Vec<u8> {

		let mut rc=Vec::new();
		let idx=self.find_dealer_by_addr(&dealer_addr);
		if idx==self.dealers_map().len() {
			return rc;
		}

		let dealer=self.dealers_map().get(idx);
		for miner
		in dealer.miners.iter() {
			rc.append(&mut miner.to_address().to_vec());
			// let ipfs=self.ipfs_map().get(miner).unwrap();
			// rc.append(&mut ipfs.to_bytes_be().to_address().to_vec());
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
			if self.addresses().get(token.miner as usize)==filter_miner {
				results[token.resp as usize]=results[token.resp as usize]+1;
			}
		}
		return results;
	}


	//retourne l'ensemble des distributeurs référencés si l'adresse est 0x0 ou les distributeurs d'un mineur
	#[view(dealers)]
	fn get_dealers(&self,filter_miner:ManagedAddress) -> Vec<u8> {
		let mut rc=Vec::new();

		for idx in 0..self.dealers_map().len() {
			let dealer=self.dealers_map().get(idx);
			if filter_miner.is_zero() || dealer.miners.contains(&filter_miner) {
				rc.append(&mut dealer.addr.to_address().to_vec());
				rc.push(dealer.state);
			}
		}

		return rc;
	}



		//Permet d'ajouter un distributeur à la liste des distributeurs du token
	//Cette fonction est réservé au propriétaire du token
	#[endpoint]
	fn add_dealer(&self,  token_id: u64, addr: ManagedAddress) -> SCResult<()> {
		let caller=self.blockchain().get_caller();
		let mut token = self.tokens_map().get(token_id as usize);

		require!(token_id < self.get_total_minted(), "E20: Token does not exist!");
		require!(self.addresses().get(token.owner as usize) == caller,"E21: Only token owner can add dealer");

		let dealer_id = self.find_dealer_by_addr(&addr);
		require!(dealer_id < self.dealers_map().len() ,"Distributeur non reference");

		let dealer=self.dealers_map().get(dealer_id);

		//Recherche du mineur du token dans la whitelist du dealer
		for miner_addr in dealer.miners.iter() {
			if miner_addr==&self.addresses().get(token.miner as usize) {
				//On ajoute le nouveau dealer au token
				token.dealer_ids.push(dealer_id as u32);
				token.dealer_markup.push(0u16);
				token.dealers=token.dealers+1;
				self.tokens_map().set(token_id as usize,&token);
				return Ok(())
			}
		}

		sc_error!("le miner du token n'est pas autorisé par le dealer")
	}


	//efface l'ensemble des distributeurs
	#[endpoint]
	fn clear_dealer(&self,  token_id: u64) -> SCResult<()> {
		let mut token = self.tokens_map().get(token_id as usize);

		let caller=self.blockchain().get_caller();

		require!(token_id < self.get_total_minted(), "E22: Token does not exist!");
		require!(self.addresses().get(token.owner as usize) == caller,"E23: Only token owner can remove dealer");

		token.dealer_ids=Vec::new();
		token.dealer_markup=Vec::new();
		token.dealers=0u8;

		self.tokens_map().set(token_id as usize,&token);

		return Ok(())
	}



	//Modifier le prix (dans la fourchette initialement défini par le mineur du token)
	//Seul les distributeurs peuvent modifier le prix
	#[endpoint]
	fn price(&self, token_id: u64, markup: u16) -> SCResult<()> {
		require!(token_id < self.get_total_minted(), "E24: Token does not exist!");
		let mut token = self.tokens_map().get(token_id as usize);

		let caller = self.blockchain().get_caller();

		let addrs=self.get_dealer_addresses_for_token(&token);
		let idx = addrs.iter().position(|x| x == &caller).unwrap_or(1000);

		require!(idx<1000, "E25: Modif de prix uniquement pour les distributeurs");
		require!(markup <= token.max_markup,"E26: Vous ne pouvez pas augmenter autant le prix");
		require!(markup >= token.min_markup,"E27: Vous ne pouvez pas réduire autant le prix");

		token.dealer_markup[idx] = markup;
		self.tokens_map().set(token_id as usize,&token);

		return Ok(());
	}



	//Fonction d'achat d'un token
	//token_id: désigne le token à acheter
	//dealer: déclare le vendeur qui à fait la vente. Cela permet au système de récupéré le prix avec la commission et de procéder au reversement
	//Voir l'exemple de la fonction fund dans https://github.com/ElrondNetwork/elrond-wasm-rs/blob/master/contracts/examples/crowdfunding-esdt/src/crowdfunding_esdt.rs
	#[payable("EGLD")]
	#[endpoint]
	fn buy(&self, #[payment] payment: BigUint,token_id: u64,dealer:ManagedAddress) -> SCResult<()> {

		//let (payment, _pay_token)=self.call_value().payment_token_pair();

		require!(token_id < self.get_total_minted(), "E28: Ce token n'existe pas");
		let mut token = self.tokens_map().get(token_id as usize);

		let caller=self.blockchain().get_caller();
		require!(self.addresses().get(token.owner as usize) != caller,"E29: Ce token vous appartient déjà");
		require!(token.properties & FOR_SALE>0,"E30: Ce token n'est pas en vente");

		let addrs=self.get_dealer_addresses_for_token(&token);
		let idx = addrs.iter().position(|x| x == &dealer).unwrap_or(1000);

		let mut payment_for_dealer=0u64;
		if idx<1000 {
			payment_for_dealer=10000000000000000*token.dealer_markup[idx] as u64;
		}

		require!(token.properties & DIRECT_SELL>0 || !dealer.is_zero() ,"E31: La vente directe n'est pas autorisé");
		require!(dealer.is_zero() || idx<1000 ,"E32: Le revendeur n'est pas autorisé");

		if(token.properties & UNIK>0){
			//TODO ajouter ici le code
			//let tokens=self.get_tokens(ManagedAddress::zero(),caller,ManagedAddress::zero());
			//require!(!self.is_in(&token,tokens),"Ce token ne peut être acheté qu'une seule fois");
		}

		//calcul du payment au owner
		let mut payment_for_owner=payment-BigUint::from(payment_for_dealer);
		//Dans le cas d'un ESDT on corrige la valeur de payment en attendant de savoir comment la passer en argument depuis python
		let money=self.get_esdt(&token);
		if money.is_esdt() {
			payment_for_owner=BigUint::from(10000000000000000*token.price as u64)-BigUint::from(payment_for_dealer);
		} else {
			require!(payment_for_owner >= BigUint::from(token.price.clone() as u64),"E33: Paiement du propriétaire inferieur au prix du token");
		}

		if !dealer.is_zero() && payment_for_dealer>0 {
			//On retribue le mineur sur la commission du distributeur
			if token.miner_ratio>0 {
				let payment_for_miner=1000000000000*token.dealer_markup[idx] as u64*token.miner_ratio as u64;
				self.send_money(&token,&self.addresses().get(token.miner as usize),BigUint::from(payment_for_miner),b"miner pay");
				payment_for_dealer=payment_for_dealer-payment_for_miner;
			}

			//Transaction issue d'un revendeur
			self.send_money(&token,&dealer,BigUint::from(payment_for_dealer),b"dealer pay");
		}

		if payment_for_owner > BigUint::from(0u64) {
			self.send_money(&token,&self.addresses().get(token.owner as usize),payment_for_owner,b"owner pay");
		}

		token.properties=&token.properties & !FOR_SALE;//Le token n'est plus a vendre
		token.owner=self.set_addresses(&caller); //On change le propriétaire
		self.tokens_map().set(token_id as usize,&token);

		return Ok(());
	}




	fn get_dealer_addresses_for_token(&self,token: &Token) -> Vec<ManagedAddress> {
		let mut rc=Vec::new();
		for i in 0..token.dealer_ids.len(){
			let dealer=self.dealers_map().get(token.dealer_ids[i] as usize);
			rc.push(dealer.addr);
		}
		return rc;
	}



	fn vec_equal(&self,va: &Vec<u8>, vb: &Vec<u8>) -> bool {
		if va.len()!= vb.len() {return false};
		for i in 0..va.len() {
			if va[i]!=vb[i] {return false;}
		}
		return true;
	}


	//Complete la réference par la chaine complete
	fn complete_token(&self,id: u64) -> Token {
		return self.tokens_map().get(id as usize);
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


		//Tag /nfts get_nfts tokens
	//Récupérer l'ensemble des tokens en appliquant les filtres sauf si celui est à la valeur 0x0
	//seller: uniquement les tokens dont "seller" fait parti des distributeurs déclarés
	//owner: uniquement les tokens dont le propriétaire est "owner"
	//miner: uniquement les tokens fabriqués par "miner"
	#[view(tokens)]
	fn get_tokens(&self,seller_filter: ManagedAddress,owner_filter: ManagedAddress, miner_filter: ManagedAddress) -> Vec<Vec<u8>> {
		let mut rc=Vec::new();

		let total_minted = self.get_total_minted();

		let idx_owner_filter=self.set_addresses(&owner_filter);
		let idx_seller_filter=self.set_addresses(&seller_filter);
		let idx_miner_filter=self.set_addresses(&miner_filter);

		for i in 0..total_minted {
			let mut token=self.complete_token(i);

			let idx = self.find_dealer_in_token(idx_seller_filter,&token);

			if 		(idx_owner_filter==0u32 || idx_owner_filter == token.owner)
				&& 	(idx_miner_filter==0u32 || idx_miner_filter == token.miner)
				&& 	(idx_seller_filter==0u32 || idx!=NOT_FIND ) {

				let description=self.strs().get(token.description as usize);
				let collection=self.strs().get(token.collection as usize);

				let token_owner_addr=self.addresses().get(token.owner as usize);
				let token_miner_addr=self.addresses().get(token.miner as usize);

				let mut item:Vec<u8>=Vec::new();

				//On commence par inscrire la taille de token_price & title dont les tailles dépendent du contenu
				//doc sur le conversion :https://docs.rs/elrond-wasm/0.10.3/elrond_wasm/
				item.append(&mut collection.len().to_be_bytes().to_vec());
				item.append(&mut description.len().to_be_bytes().to_vec());
				item.append(&mut self.get_esdt(&token).as_name().len().to_be_bytes().to_vec());

				//Puis on ajoute l'ensemble des informations d'un token
				//dans un vecteur d'octets
				let mut price=token.price;
				let mut markup=0u16;
				if idx< token.dealer_ids.len()  {
					price=price+100*token.dealer_markup[idx] as u32;
					markup=token.dealer_markup[idx];
				}

				let mut has_secret=0u8;
				if token.secret>0 || token.gift>0 {
					has_secret=1u8;
				}

				item.append(&mut price.to_be_bytes().to_vec());
				item.append(&mut self.get_esdt(&token).as_name().into_vec());

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

				item.append(&mut token.min_markup.to_be_bytes().to_vec());
				item.append(&mut token.max_markup.to_be_bytes().to_vec());
				item.append(&mut markup.to_be_bytes().to_vec());
				item.append(&mut token.miner_ratio.to_be_bytes().to_vec());
				item.append(&mut token_miner_addr.to_address().to_vec());
				item.append(&mut i.to_be_bytes().to_vec()); //Identifiant du token
				item.append(&mut token.collection.to_be_bytes().to_vec());
				item.append(&mut collection.to_vec());
				item.append(&mut description.to_vec());

				rc.push(item);
			}

		}
		return rc;
	}



	#[view(contractOwner)]
	#[storage_get("owner")]
	fn get_owner(&self) -> ManagedAddress;
	#[storage_set("owner")]
	fn set_owner(&self, owner: &ManagedAddress);




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
	#[storage_mapper("dealers_map")]
	fn dealers_map(&self) -> VecMapper<Dealer<Self::Api>>;

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

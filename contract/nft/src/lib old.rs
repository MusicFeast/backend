/*!
Non-Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::non_fungible_token::core::{
    NonFungibleTokenCore, NonFungibleTokenResolver
};

//use near_contract_standards::non_fungible_token::approval::ext_nft_approval_receiver;


use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{TokenId};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise,
    Balance, serde_json::json, assert_one_yocto, Gas, ext_contract, PromiseOrValue,
};

use near_sdk::collections::{LazyOption, UnorderedMap, UnorderedSet};

/* custon codigo */
use near_sdk::json_types::{/*ValidAccountId,*/ U128, /*U64*/};

use serde::Serialize;
use serde::Deserialize;
use std::collections::HashMap;
use near_sdk::env::is_valid_account_id;
pub mod event;
pub use event::NearEvent;


pub const TOKEN_DELIMETER: char = ':';
pub const TITLE_DELIMETER: &str = " #";
pub const VAULT_FEE: u128 = 500;

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(10_000_000_000_000);
const GAS_FOR_NFT_TRANSFER_CALL: Gas = Gas(40_000_000_000_000); //GAS_FOR_NFT_TRANSFER_CALL(30_000_000_000_000) + GAS_FOR_RESOLVE_TRANSFER;
//const GAS_FOR_NFT_APPROVE: Gas = Gas(10_000_000_000_000);
//const GAS_FOR_MINT: Gas = Gas(90_000_000_000_000);
//const NO_DEPOSIT: Balance = 0;
const MAX_PRICE: Balance = 1_000_000_000 * 10u128.pow(24);


pub type TokenSeriesId = String;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}


/* codigo customizado */

#[ext_contract(ext_non_fungible_token_receiver)]
trait NonFungibleTokenReceiver {
    /// Returns `true` if the token should be returned back to the sender.
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> Promise;
}

#[ext_contract(ext_self)]
trait NonFungibleTokenResolverExt {
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool;
}



#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenSeries {
    desc_series: String,
	metadata: TokenMetadata,
	creator_id: AccountId,
	tokens: UnorderedSet<TokenId>,
    objects_mint: UnorderedSet<String>,
    price: Option<Balance>,
    price_usd: Option<f64>,
    is_mintable: bool,
    royalty: HashMap<AccountId, u32>,
    royalty_buy: HashMap<String, u32>
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RoyaltyBuy {
	artist_id: String,
	porcentaje: String,
    amount: String,
    tax: String
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenSeriesJson {
    token_series_id: TokenSeriesId,
	metadata: TokenMetadata,
	creator_id: AccountId,
    royalty: HashMap<AccountId, u32>
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenSeriesJson2 {
	token_series_id: TokenSeriesId,
    metadata: TokenMetadata,
	creator_id: AccountId,
    price: Option<Balance>,
    is_mintable: bool,
    royalty: HashMap<AccountId, u32>
}


#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokensView {
	owner_id: String,
    token_id: String
}


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ArtistObject {
	id: u128,
    name: String,
    next_collection: i128,
}


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TypeTokenObject {
	id: u128,
    description: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RestrictionsObject {
	id: String,
    token_id_s: u128,
    token_id_r: u128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenCustom {
    token_id: TokenId,
    owner_id: AccountId,
    metadata: Option<TokenMetadata>,
    approved_account_ids: Option<HashMap<AccountId, u64>>,
    royalty: Option<HashMap<AccountId, u32>>
}

/* fin codigo costumizado */

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    /* codigo costumizado */
    owner_id: AccountId,
    list_admin: UnorderedSet<AccountId>,
    list_vip: UnorderedSet<AccountId>,
    maestro_artist: UnorderedMap<u128, ArtistObject>,
    id_artist: u128,
    id_type_token_series: u128,
    type_token_series: UnorderedMap<u128, TypeTokenObject>,
    id_objects: u128,
    id_serie: u128,
    token_series_by_id: UnorderedMap<TokenSeriesId, TokenSeries>,
    restrictions: UnorderedMap<String, RestrictionsObject>,
    vault_id: AccountId,
    black_list_reedemer: UnorderedSet<String>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    /*codigo costumizado*/
    AdminKey,
    VipKey,
    ArtistKey,
    TypeTokenSeriesKey,
    RestrictionsKeys,
    TokenSeriesById,
    TokensBySeriesInner { token_series: String },
    TokensByObjectsInner { token_series: String },
    BlackListReedemerKey,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId, vault_id: AccountId,) -> Self {
        Self::new(
            owner_id,
            vault_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "Music Feast".to_string(),
                symbol: "MusicFeast".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    #[init]
    pub fn new(owner_id: AccountId, vault_id: AccountId, metadata: NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id.clone(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            /* codigo costumizado */
            owner_id: owner_id,
            list_admin: UnorderedSet::new(StorageKey::AdminKey),
            list_vip: UnorderedSet::new(StorageKey::VipKey),
            maestro_artist: UnorderedMap::new(StorageKey::ArtistKey),
            id_artist: 0,
            id_type_token_series: 0,
            type_token_series: UnorderedMap::new(StorageKey::TypeTokenSeriesKey),
            id_objects: 0,
            id_serie: 0,
            token_series_by_id: UnorderedMap::new(StorageKey::TokenSeriesById),
            restrictions: UnorderedMap::new(StorageKey::RestrictionsKeys),
            vault_id: vault_id,
            black_list_reedemer: UnorderedSet::new(StorageKey::BlackListReedemerKey),
        }
    }

    /* codigo original */
    /*
    /// Mint a new token with ID=`token_id` belonging to `receiver_id`.
    ///
    /// Since this example implements metadata, it also requires per-token metadata to be provided
    /// in this call. `self.tokens.mint` will also require it to be Some, since
    /// `StorageKey::TokenMetadata` was provided at initialization.
    ///
    /// `self.tokens.mint` will enforce `predecessor_account_id` to equal the `owner_id` given in
    /// initialization call to `new`.
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        receiver_id: AccountId,
        token_metadata: TokenMetadata,
    ) -> Token {
        self.tokens.mint(token_id, receiver_id, Some(token_metadata))
    }*/

    /* codigo custom */
    pub fn edit_vault(&mut self, account_id: AccountId){
        assert!(self.owner_id == env::signer_account_id() || self.list_admin.contains(&env::signer_account_id()), "Only administrator");
        self.vault_id = account_id;
    }

    // cargar usuarios a la lista de administradores
    // solo los administradores pueden usar esta funcion
    pub fn add_admin(&mut self, account_id: AccountId) {
        assert!(self.owner_id == env::signer_account_id() || self.list_admin.contains(&env::signer_account_id()), "Only administrator");
        self.list_admin.insert(&account_id.clone());

        env::log_str(
            &json!({
                "type": "add_admin",
                "params": {
                    "account_id": account_id.to_string()
                }
            })
            .to_string(),
        );

    }

    // cargar usuarios a la lista de administradores
    // solo los administradores pueden usar esta funcion
    pub fn add_vip(&mut self, account_id: AccountId) {
        assert!(self.owner_id == env::signer_account_id() || self.list_admin.contains(&env::signer_account_id()), "Only administrator");
        self.list_vip.insert(&account_id.clone());

        env::log_str(
            &json!({
                "type": "add_vip",
                "params": {
                    "account_id": account_id.to_string()
                }
            })
            .to_string(),
        );

    }

    pub fn is_vip(&self, account_id: AccountId) -> bool {
        self.list_vip.contains(&account_id)
    } 

    pub fn add_black_list_reedemer(&mut self, type_token: String) {
        assert!(self.owner_id == env::signer_account_id() || self.list_admin.contains(&env::signer_account_id()), "Only administrator");
        self.black_list_reedemer.insert(&type_token.clone());

        env::log_str(
            &json!({
                "type": "add_black_list_reedemer",
                "params": {
                    "type_token": type_token
                }
            })
            .to_string(),
        );

    }

    pub fn add_artist(&mut self, name: String) {
        assert!(self.owner_id == env::signer_account_id() || self.list_admin.contains(&env::signer_account_id()), "Only administrator");
        
        let id_artist: u128 = self.id_artist + 1;

        let data_artist = ArtistObject {
            id: id_artist,
            name: name.clone(),
            next_collection: 1
        };

        self.maestro_artist.insert(&data_artist.id, &data_artist);

        self.id_artist = id_artist;

        env::log_str(
            &json!({
                "type": "add_artist",
                "params": {
                    "id": id_artist.to_string(),
                    "name": name,
                    "collection": 1,
                }
            })
            .to_string(),
        );
    }

    pub fn get_artist(self) -> Vec<ArtistObject> {
        self.maestro_artist.iter().map(|(_k, v)| { v }).collect()
    }


    pub fn add_type_token_series(&mut self, description: String) {
        assert!(self.owner_id == env::signer_account_id() || self.list_admin.contains(&env::signer_account_id()), "Only administrator");
        
        let id_type_token: u128 = self.id_type_token_series + 1;

        let data_type_token = TypeTokenObject {
            id: id_type_token,
            description: description.clone(),
        };

        self.type_token_series.insert(&data_type_token.id, &data_type_token);

        self.id_type_token_series = id_type_token;

        env::log_str(
            &json!({
                "type": "add_type_token_series",
                "params": {
                    "id": id_type_token.to_string(),
                    "description": description
                }
            })
            .to_string(),
        );
    }


    pub fn get_type_token_series(self) -> Vec<TypeTokenObject> {
        self.type_token_series.iter().map(|(_k, v)| {v}).collect()
    }


    pub fn add_restriction(&mut self, token_id_s: u128, token_id_r: u128) {
        assert!(self.owner_id == env::signer_account_id() || self.list_admin.contains(&env::signer_account_id()), "Only administrator");
        assert!(self.type_token_series.get(&token_id_s).is_some(), "Token_id_s not exist");
        assert!(self.type_token_series.get(&token_id_r).is_some(), "Token_id_r not exist");
        assert!(self.restrictions.get(&token_id_s.to_string()).is_none(), "The requested token already has a constraint.");

        let data_restriction = RestrictionsObject {
            id: token_id_s.to_string(),
            token_id_s: token_id_s,
            token_id_r: token_id_r,
        };

        self.restrictions.insert(&data_restriction.id, &data_restriction);

        env::log_str(
            &json!({
                "type": "add_restriction",
                "params": {
                    "token_id_s": token_id_s.to_string(),
                    "token_id_r": token_id_r.to_string()
                }
            })
            .to_string(),
        );
    }

    pub fn get_restriction(self) -> Vec<RestrictionsObject> {
        self.restrictions.iter().map(|(_k, v)| {v}).collect()
    }

    pub fn next_collection(&mut self, artist_id: u128) {
        assert!(self.owner_id == env::signer_account_id() || self.list_admin.contains(&env::signer_account_id()), "Only administrator");
        
        let mut artist = self.maestro_artist.get(&artist_id).expect("artist not exist");

        artist.next_collection += 1;

        self.maestro_artist.insert(&artist_id, &artist);

        env::log_str(
            &json!({
                "type": "next_collection",
                "params": {
                    "artist_is": artist_id.to_string(),
                    "next_collection": artist.next_collection.to_string()
                }
            })
            .to_string(),
        );
    }

   #[payable]
    pub fn update_nft_series(&mut self, 
        token_series_id: TokenSeriesId, 
        title: Option<String>,
        description: Option<String>,
        media: Option<String>,
    ) -> TokenSeriesJson {
        assert!(self.owner_id == env::signer_account_id() || self.list_admin.contains(&env::signer_account_id()), "Only administrator");

        //let initial_storage_usage = env::storage_usage();
        

        let mut nft_serie = self.token_series_by_id.get(&token_series_id).expect("tonken serie id not exist");
        if title.is_some() { nft_serie.metadata.title = title; }
        if description.is_some() { nft_serie.metadata.description = description; }
        if media.is_some() { nft_serie.metadata.media = media; }

        self.token_series_by_id.insert(&token_series_id, &nft_serie);

        env::log_str(
            &json!({
                "type": "update_nft_series",
                "params": {
                    "token_series_id": token_series_id,
                    "desc_series": nft_serie.desc_series.clone(),
                    "token_metadata": nft_serie.metadata.clone(),
                    "creator_id": nft_serie.creator_id.clone(),
                    "price": nft_serie.price.unwrap().to_string(),
                    "royalty": nft_serie.royalty.clone()
                }
            })
            .to_string(),
        );

        //refund_deposit(env::storage_usage() - initial_storage_usage, 0);


        TokenSeriesJson {
            token_series_id,
			metadata: nft_serie.metadata.clone(),
			creator_id: nft_serie.creator_id.clone(),
            royalty: nft_serie.royalty,
		}
    }
    pub fn prueba() {
        let price: f64 = 4;
        let tasa: f64 = 2;
        let price_near: f64 = price / tasa;
        let yocto_near: u18 = 1000000000000000000000000;
        let price_yocto: u128 = (price_near * yocto_near) as u128;
        env::log_str(price_yocto.to_string());
    }

    #[payable]
    pub fn nft_series(
        &mut self,
        artist_id: u128,
        type_token_id: u128,
        objects: bool,
        token_metadata: TokenMetadata,
        objects_ids: Option<Vec<String>>,
        price: Option<U128>,
        royalty: Option<HashMap<AccountId, u32>>,
        royalty_buy: Option<HashMap<String, u32>>,
    ) -> TokenSeriesJson {
        assert!(self.owner_id == env::signer_account_id() || self.list_admin.contains(&env::signer_account_id()), "Only administrator");
        let data_artist = self.maestro_artist.get(&artist_id).expect("Artist_id not exist");
        
        let type_token = self.type_token_series.get(&type_token_id).expect("type_token_id not exist");

        let initial_storage_usage = env::storage_usage();
        let caller_id = env::signer_account_id();

        let mut token_series_id: String;

        if objects {
            self.id_objects += 1;
            token_series_id = format!("{}|{}|{}|{}", artist_id, type_token_id, data_artist.next_collection, self.id_objects.to_string());
        } else {
            token_series_id = format!("{}|{}|1", artist_id, type_token_id);
            if type_token_id > 1 && type_token_id <= 6 {
                token_series_id = format!("{}|{}|{}", artist_id, type_token_id, data_artist.next_collection);
            } else if type_token_id > 6 {
                token_series_id = format!("{}|{}|{}|{}", artist_id, type_token_id, data_artist.next_collection, self.id_serie.to_string());
            }
        }

        assert!(
            self.token_series_by_id.get(&token_series_id).is_none(),
            "duplicate token_series_id"
        );

        let title = token_metadata.title.clone();
        assert!(title.is_some(), "token_metadata.title is required");
        
        
        let mut total_perpetual = 0;
        let mut total_accounts = 0;
        let royalty_res_buy =  if let Some(royalty_buy) = royalty_buy {
            for (k, v) in royalty_buy.iter() {
                if self.maestro_artist.get(&*k.parse::<u128>().as_ref().unwrap()).is_none() && *k.parse::<u128>().as_ref().unwrap() > 0 {
                    env::panic_str("Not valid artist_id for royalty");
                };
                total_perpetual += v;
                total_accounts += 1;
            }
            royalty_buy
        } else {
            HashMap::new()
        };

        assert!(total_accounts <= 10, "royalty_buy exceeds 10 accounts");

        assert!(
            total_perpetual <= 10000,
            "Exceeds maximum royalty_buy -> 10000",
        );


        total_perpetual = 0;
        total_accounts = 0;
        let royalty_res: HashMap<AccountId, u32> = if let Some(royalty) = royalty {
            for (k , v) in royalty.iter() {
                if !is_valid_account_id(k.as_bytes()) {
                    env::panic_str("Not valid account_id for royalty");
                };
                total_perpetual += *v;
                total_accounts += 1;
            }
            royalty
        } else {
            HashMap::new()
        };

        assert!(total_accounts <= 10, "royalty exceeds 10 accounts");

        assert!(
            total_perpetual <= 9000,
            "Exceeds maximum royalty -> 9000",
        );

        let price_res: Option<u128> = if price.is_some() {
            assert!(
                price.unwrap().0 < MAX_PRICE,
                "price higher than {}",
                MAX_PRICE
            );
            Some(price.unwrap().0)
        } else {
            None
        };

        if objects_ids.is_some() && objects == false {
            for item in objects_ids.clone().unwrap().iter() {
                assert!(item.split("|").next().unwrap().to_string() == artist_id.to_string(), "The object does not belong to the selected artist");
                self.token_series_by_id.get(&item).expect("token objects series id not exist");
            }
        }

        self.token_series_by_id.insert(&token_series_id, &TokenSeries{
            desc_series: type_token.description.clone(),
            metadata: token_metadata.clone(),
            creator_id: caller_id.clone(),
            tokens: UnorderedSet::new(
                StorageKey::TokensBySeriesInner {
                    token_series: token_series_id.clone(),
                }
                .try_to_vec()
                .unwrap(),
            ),
            objects_mint: UnorderedSet::new(
                StorageKey::TokensByObjectsInner {
                    token_series: token_series_id.clone(),
                }
                .try_to_vec()
                .unwrap(),
            ),
            price: price_res,
            price_usd: Some(4.1),
            is_mintable: true,
            royalty: royalty_res.clone(),
            royalty_buy: royalty_res_buy.clone(),
        });

        if objects_ids.is_some() && objects == false {
            let mut data_serie = self.token_series_by_id.get(&token_series_id).expect("token series id no existe");
            for item in objects_ids.unwrap().iter() {
                data_serie.objects_mint.insert(&item.to_string());
            }   
            self.token_series_by_id.insert(&token_series_id, &data_serie);
        }
        

        env::log_str(
            &json!({
                "type": "nft_create_series",
                "params": {
                    "token_series_id": token_series_id.to_string(),
                    "objects": objects,
                    "desc_series": type_token.description.to_string(),
                    "token_metadata": token_metadata,
                    "creator_id": caller_id.to_string(),
                    "price": price,
                    "royalty": royalty_res
                }
            })
            .to_string(),
        );

        refund_deposit(env::storage_usage() - initial_storage_usage, 0);

		TokenSeriesJson{
            token_series_id,
			metadata: token_metadata,
			creator_id: caller_id.into(),
            royalty: royalty_res,
		}
    }


    #[payable]
    pub fn nft_buy(
        &mut self, 
        token_series_id: TokenSeriesId
    ) -> TokenId {
        //token_series_id.split("|").collect::<Vec<&str>>()[2];
        
        /*account_id: TokenSeriesId,
        artist_id: TokenMetadata,
        porcentaje: u32,*/

        let token_series = self.token_series_by_id.get(&token_series_id).expect("Token series not exist");
        let price: u128 = token_series.price.expect("not for sale");
        let attached_deposit = env::attached_deposit();
        let receiver_id: AccountId = env::signer_account_id();
        let type_token = token_series_id.split("|").collect::<Vec<&str>>()[1].to_string();

        if self.list_vip.contains(&env::signer_account_id()) && type_token == "1" {
            assert!(
                attached_deposit >= 300000000000000000000000,
                "attached deposit is less than price : {}",
                "300000000000000000000000"    
            );

            let token_id: TokenId = self._nft_mint_series(token_series_id, receiver_id.clone());

            for item in token_series.objects_mint.iter() {
                self._nft_mint_series(item.to_string(), receiver_id.clone());
            }

            token_id
        } else {    
            assert!(
                attached_deposit >= price,
                "attached deposit is less than price : {}",
                price
            );

            let artist_id = token_series_id.split("|").next().unwrap().to_string();

            let restriction = self.restrictions.get(&token_series.metadata.reference.unwrap().to_string());
            
            if restriction.is_some() {
                let tokens_per_owner = self.tokens.tokens_per_owner.as_ref().expect(
                    "Could not find tokens_per_owner when calling a method on the enumeration standard.",
                );
                let token_set = tokens_per_owner.get(&receiver_id).expect("You do not have the corresponding token to buy the requested token");
                let token_id_r: String = restriction.unwrap().token_id_r.to_string();
                
                let valit_token_owner = token_set
                    .iter()
                    .find(|token_id| token_id.split(":").next().unwrap().to_string() == format!("{}|{}|1",artist_id, token_id_r).to_string());
                assert!(valit_token_owner != None
                    , "You do not have the corresponding token to buy the requested token");
            }

            let token_id: TokenId = self._nft_mint_series(token_series_id, receiver_id.clone());

            for item in token_series.objects_mint.iter() {
                self._nft_mint_series(item.to_string(), receiver_id.clone());
            }

            //let for_vault = price as u128 * VAULT_FEE / 10_000u128;
            //let price_deducted = price - for_vault;
            //Promise::new(token_series.creator_id).transfer(price_deducted);
            
            Promise::new(self.vault_id.clone()).transfer(price);

            let amount_final: u128 = price - (price * 50)/10000;
            let tax: u128 = (price * 50)/10000;

            let royalty_buy: Vec<RoyaltyBuy> = token_series.royalty_buy.iter().map(|(k, v)| 
                {
                    let mut tax_final: String = "0".to_string();
                    if *k.parse::<u128>().as_ref().unwrap() == 0 || *k == artist_id {
                        tax_final = (tax/2).to_string();
                    }
                    RoyaltyBuy{
                        artist_id: k.clone(),
                        porcentaje: v.to_string(),
                        amount: ((amount_final * (*v as u128))/10000).to_string(),
                        tax: tax_final,
                    }
                }).collect();
            

            env::log_str(
                &json!({
                    "type": "nft_buy",
                    "params": {
                        "artista": artist_id.clone(),
                        "price": price.to_string(),
                        "amount_artist": ((amount_final * 7000)/10000).to_string(), // 70% del artista
                        "amount_musicfeast": ((amount_final * 3000)/10000).to_string(), // 30% musicfeast
                        "tax_artist": (tax/2).to_string(),
                        "tax_musicfeast": (tax/2).to_string(),
                        "royalty": royalty_buy,
                    }
                })
                .to_string(),
            );

            token_id
        }
    }

    pub fn auto_swap_ini(
        &mut self, 
        artist_id: String, 
        amount_near: U128,
        tax_near: U128,
        ft_token: String,
    ) {
        //assert_eq!(env::predecessor_account_id(), self.vault_id.clone(), "no autorizado");
        
        env::log_str(
            &json!({
                "type": "auto_swap_ini",
                "params": {
                    "artista": artist_id.clone(),
                    "amount_near": amount_near.0.to_string(),
                    "tax_near": tax_near.0.to_string(),
                    "ft_token": ft_token,
                }
            })
            .to_string(),
        );
    }

    pub fn auto_swap_end(
        &mut self, 
        artist_id: String,
        amount_near: U128,
        tax_near: U128,
        amount_usd: U128,
        ft_token: String,
    ) {
        //assert_eq!(env::predecessor_account_id(), self.vault_id.clone(), "no autorizado");
        
        env::log_str(
            &json!({
                "type": "auto_swap_end",
                "params": {
                    "artista": artist_id.clone(),
                    "amount_near": amount_near.0.to_string(),
                    "tax_near": tax_near.0.to_string(),
                    "amount_usd": amount_usd.0.to_string(),
                    "ft_token": ft_token,
                }
            })
            .to_string(),
        );
    }

    pub fn auto_swap_error(
        &mut self, 
        artist_id: String, 
        amount_near: U128,
        tax_near: U128,
        arg: String,
    ) {
        //assert_eq!(env::predecessor_account_id(), self.vault_id.clone(), "no autorizado");
        
        env::log_str(
            &json!({
                "type": "auto_swap_error",
                "params": {
                    "artista": artist_id.clone(),
                    "amount_near": amount_near.0.to_string(),
                    "tax_near": tax_near.0.to_string(),
                    "arg": arg
                }
            })
            .to_string(),
        );
    }


    #[payable]
    pub fn nft_mint(
        &mut self, 
        token_series_id: TokenSeriesId, 
        receiver_id: AccountId
    ) -> TokenId {
        let token_series = self.token_series_by_id.get(&token_series_id).expect("Token series not exist");
        assert_eq!(env::predecessor_account_id(), token_series.creator_id.clone(), "not creator");
        let token_id: TokenId = self._nft_mint_series(token_series_id, receiver_id.clone());

        token_id
    }

    
    
    /*#[payable]
    pub fn nft_mint_and_approve(
        &mut self, 
        token_series_id: TokenSeriesId, 
        account_id: AccountId,
        msg: Option<String>,
    ) -> Option<Promise> {
        let initial_storage_usage = env::storage_usage();

        let token_series = self.token_series_by_id.get(&token_series_id).expect("Token series not exist");
        assert_eq!(env::predecessor_account_id(), token_series.creator_id.clone(), "not creator");
        let token_id: TokenId = self._nft_mint_series(token_series_id, token_series.creator_id.clone());

        let approvals_by_id = self.tokens.approvals_by_id.as_mut().unwrap();

        let approved_account_ids =
            &mut approvals_by_id.get(&token_id).unwrap_or_else(|| HashMap::new());
        let account_id: AccountId = account_id.into();
        let approval_id: u64 =
            self.tokens.next_approval_id_by_id.as_ref().unwrap().get(&token_id).unwrap_or_else(|| 1u64);
        approved_account_ids.insert(account_id.clone(), approval_id);

        approvals_by_id.insert(&token_id, &approved_account_ids);

        self.tokens.next_approval_id_by_id.as_mut().unwrap().insert(&token_id, &(approval_id + 1));

        refund_deposit(env::storage_usage() - initial_storage_usage, 0);

        NearEvent::log_nft_mint(
            token_series.creator_id.to_string(),
            vec![token_id.clone()],
            None,
        );

        if let Some(msg) = msg {
            Some(ext_nft_approval_receiver::ext(account_id)
            .with_static_gas(env::prepaid_gas() - GAS_FOR_NFT_APPROVE - GAS_FOR_MINT)
            .nft_on_approve(token_id, token_series.creator_id, approval_id, msg))
            
            /*Some(ext_approval_receiver::nft_on_approve(
                token_id,
                token_series.creator_id,
                approval_id,
                msg,
                &account_id,
                NO_DEPOSIT,
                env::prepaid_gas() - GAS_FOR_NFT_APPROVE - GAS_FOR_MINT,
            ))*/
        } else {
            None
        }
    }*/


    #[payable]
    pub fn put_nft_series_price(&mut self, token_series_id: TokenSeriesId, price: Option<U128>) -> Option<U128> {
        assert_one_yocto();

        let mut token_series = self.token_series_by_id.get(&token_series_id).expect("Token series not exist");
        assert_eq!(
            env::predecessor_account_id(),
            token_series.creator_id,
            "Creator only"
        );

        assert_eq!(
            token_series.is_mintable,
            true,
            "Token series is not mintable"
        );

        if price.is_none() {
            token_series.price = None;
        } else {
            assert!(
                price.unwrap().0 < MAX_PRICE,
                "Price higher than {}",
                MAX_PRICE
            );
            token_series.price = Some(price.unwrap().0);
        }

        self.token_series_by_id.insert(&token_series_id, &token_series);

        return price;
    }


    fn _nft_mint_series(
        &mut self, 
        token_series_id: TokenSeriesId, 
        receiver_id: AccountId
    ) -> TokenId {
        let mut token_series = self.token_series_by_id.get(&token_series_id).expect("Token series not exist");
    
        assert!(
            token_series.is_mintable,
            "Token series is not mintable"
        );

        let num_tokens = token_series.tokens.len();
        let max_copies = token_series.metadata.copies.unwrap_or(u64::MAX);
        
        assert!(num_tokens < max_copies, "Series supply maxed");

        if (num_tokens + 1) >= max_copies {
            token_series.is_mintable = false;
            token_series.price = None;
        }
        
        let token_id = format!("{}{}{}", &token_series_id, TOKEN_DELIMETER, num_tokens + 1);
        token_series.tokens.insert(&token_id);
        self.token_series_by_id.insert(&token_series_id, &token_series);
        let title: String = format!("{} {} {} {} {}", token_series.metadata.title.unwrap().clone(), TITLE_DELIMETER, token_series_id, TITLE_DELIMETER, (num_tokens + 1).to_string());
        
        let token_metadata = Some(TokenMetadata {
            title: Some(title),          
            description: token_series.metadata.description.clone(),   
            media: token_series.metadata.media.clone(),
            media_hash: token_series.metadata.media_hash, 
            copies: token_series.metadata.copies, 
            issued_at: Some(env::block_timestamp().to_string()), 
            expires_at: token_series.metadata.expires_at, 
            starts_at: token_series.metadata.starts_at, 
            updated_at: token_series.metadata.updated_at, 
            extra: token_series.metadata.extra.clone(), 
            reference: token_series.metadata.reference.clone(),
            reference_hash: token_series.metadata.reference_hash, 
        });

        let token_owner_id: AccountId = receiver_id;
      
        self.tokens.internal_mint(token_id.clone(), token_owner_id, token_metadata);

        token_id
    }



    #[payable]
    pub fn nft_burn(&mut self, token_id: TokenId, reedemer: bool) {
        assert_one_yocto();
        
        if reedemer {
            let type_token = token_id.split(":").next().unwrap().to_string().split("|").collect::<Vec<&str>>()[1].to_string();
            assert_eq!(self.black_list_reedemer.contains(&type_token), false, "El token seleccionado no se puede quemar por esta funcion");
        }

        let owner_id = self.tokens.owner_by_id.get(&token_id).unwrap();
        
        assert_eq!(
            owner_id,
            env::predecessor_account_id(),
            "Token owner only"
        );

        if let Some(next_approval_id_by_id) = &mut self.tokens.next_approval_id_by_id {
            next_approval_id_by_id.remove(&token_id);
        }

        if let Some(approvals_by_id) = &mut self.tokens.approvals_by_id {
            approvals_by_id.remove(&token_id);
        }

        if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
            let mut token_ids = tokens_per_owner.get(&owner_id).unwrap();
            token_ids.remove(&token_id);
            tokens_per_owner.insert(&owner_id, &token_ids);
        }

        if let Some(token_metadata_by_id) = &mut self.tokens.token_metadata_by_id {
            token_metadata_by_id.remove(&token_id);
        }

        self.tokens.owner_by_id.remove(&token_id);

        NearEvent::log_nft_burn(
            owner_id.to_string(),
            vec![token_id.clone()],
            None,
            None,
        );

        env::log_str(
            &json!({
                "type": "nft_burn",
                "params": {
                    "owner_id": owner_id.clone(),
                    "token_id": token_id.clone(),
                    "reedemer": reedemer
                }
            })
            .to_string(),
        );

    }


    pub fn get_nft_series_copies_availables(&self, token_series_id: TokenSeriesId) -> u64 {
		let token_series = self.token_series_by_id.get(&token_series_id).expect("Series does not exist");
        let copies_availables = token_series.metadata.copies.unwrap() - token_series.tokens.len() as u64 ;
        copies_availables    
	}

    pub fn get_nft_series_single(&self, token_series_id: TokenSeriesId) -> TokenSeriesJson {
		let token_series = self.token_series_by_id.get(&token_series_id).expect("Series does not exist");
		TokenSeriesJson{
            token_series_id,
			metadata: token_series.metadata,
			creator_id: token_series.creator_id,
            royalty: token_series.royalty,
		}
	}

    pub fn get_nft_series(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<TokenSeriesJson2> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            (self.token_series_by_id.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");

        self.token_series_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(token_series_id, token_series)| TokenSeriesJson2 {
                token_series_id: token_series_id.clone(),
                metadata: token_series.metadata,
                creator_id: token_series.creator_id,
                price: token_series.price,
                is_mintable: token_series.is_mintable,
                royalty: token_series.royalty
            })
            .collect()
    }


    pub fn nft_token(&self, token_id: TokenId) -> Option<TokenCustom> {
        let owner_id = self.tokens.owner_by_id.get(&token_id)?;
        let approved_account_ids = self
            .tokens
            .approvals_by_id
            .as_ref()
            .and_then(|by_id| by_id.get(&token_id).or_else(|| Some(HashMap::new())));

        let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
        let token_series_id = token_id_iter.next().unwrap().parse().unwrap();
        let royalty = self.token_series_by_id.get(&token_series_id).unwrap().royalty;

        let token_metadata = self.tokens.token_metadata_by_id.as_ref().unwrap().get(&token_id).unwrap();

        Some(TokenCustom {
            token_id,
            owner_id,
            metadata: Some(token_metadata),
            approved_account_ids,
            royalty: Some(royalty)
        })
    }



    pub fn nft_transfer_unsafe(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        let sender_id = env::predecessor_account_id();
        let (previous_owner_id, _) = self.tokens.internal_transfer(&sender_id, &receiver_id, &token_id, approval_id, memo.clone());

        let authorized_id : Option<String> = if sender_id != previous_owner_id {
            Some(sender_id.to_string())
        } else {
            None
        };

        NearEvent::log_nft_transfer(
            previous_owner_id.to_string(),
            receiver_id.to_string(),
            vec![token_id],
            memo,
            authorized_id,
        );
    }

    #[payable]
    pub fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        self.tokens.nft_transfer(receiver_id.clone(), token_id.clone(), approval_id, memo.clone());
    }

    #[payable]
    pub fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        let (previous_owner_id, old_approvals) = self.tokens.internal_transfer(
            &sender_id,
            &receiver_id.clone(),
            &token_id,
            approval_id,
            memo.clone(),
        );

        let authorized_id : Option<String> = if sender_id != previous_owner_id {
            Some(sender_id.to_string())
        } else {
            None
        };

        NearEvent::log_nft_transfer(
            previous_owner_id.to_string(),
            receiver_id.to_string(),
            vec![token_id.clone()],
            memo,
            authorized_id,
        );

        // Initiating receiver's call and the callback
        ext_non_fungible_token_receiver::ext(receiver_id.clone())
            .with_static_gas(env::prepaid_gas() - GAS_FOR_NFT_TRANSFER_CALL)
            .nft_on_transfer(
                sender_id,
                previous_owner_id.clone(),
                token_id.clone(),
                msg,
            ).then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .nft_resolve_transfer(
                        previous_owner_id,
                        receiver_id.into(),
                        token_id,
                        old_approvals,
                    )
            ).into()

        /*ext_non_fungible_token_receiver::nft_on_transfer(
            sender_id,
            previous_owner_id.clone(),
            token_id.clone(),
            msg,
            receiver_id.as_ref(),
            NO_DEPOSIT,
            env::prepaid_gas() - GAS_FOR_NFT_TRANSFER_CALL,
        )
        .then(ext_self::nft_resolve_transfer(
            previous_owner_id,
            receiver_id.into(),
            token_id,
            old_approvals,
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
        .into()*/

    }

    // CUSTOM enumeration standard modified here because no macro below

    pub fn nft_total_supply(&self) -> U128 {
        (self.tokens.owner_by_id.len() as u128).into()
    }

    pub fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<TokenCustom> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            (self.tokens.owner_by_id.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.tokens
            .owner_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(token_id, _)| self.nft_token(token_id).unwrap())
            .collect()
    }

    pub fn nft_supply_for_owner(self, account_id: AccountId) -> U128 {
        let tokens_per_owner = self.tokens.tokens_per_owner.expect(
            "Could not find tokens_per_owner when calling a method on the enumeration standard.",
        );
        tokens_per_owner
            .get(&account_id)
            .map(|account_tokens| U128::from(account_tokens.len() as u128))
            .unwrap_or(U128(0))
    }

    pub fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<TokenCustom> {
        let tokens_per_owner = self.tokens.tokens_per_owner.as_ref().expect(
            "Could not find tokens_per_owner when calling a method on the enumeration standard.",
        );
        let token_set = if let Some(token_set) = tokens_per_owner.get(&account_id) {
            token_set
        } else {
            return vec![];
        };
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            token_set.len() as u128 > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        token_set
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|token_id| self.nft_token(token_id).unwrap())
            .collect()
    }

    pub fn nft_payout(
        &self, 
        token_id: TokenId,
        balance: U128, 
        max_len_payout: u32
    ) -> Payout{
        let owner_id = self.tokens.owner_by_id.get(&token_id).expect("No token id");
        let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
        let token_series_id = token_id_iter.next().unwrap().parse().unwrap();
        let royalty = self.token_series_by_id.get(&token_series_id).expect("no type").royalty;

        assert!(royalty.len() as u32 <= max_len_payout, "Market cannot payout to that many receivers");

        let balance_u128: u128 = balance.into();

        let mut payout: Payout = Payout { payout: HashMap::new() };
        let mut total_perpetual = 0;

        for (k, v) in royalty.iter() {
            if *k != owner_id {
                let key = k.clone();
                payout.payout.insert(key, royalty_to_payout(*v, balance_u128));
                total_perpetual += *v;
            }
        }
        payout.payout.insert(owner_id, royalty_to_payout(10000 - total_perpetual, balance_u128));
        payout
    }

    #[payable]
    pub fn nft_transfer_payout(
        &mut self, 
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        balance: Option<U128>,
        max_len_payout: Option<u32>
    ) -> Option<Payout> {
        assert_one_yocto();
        // Transfer
        let previous_token = self.nft_token(token_id.clone()).expect("no token");
        self.tokens.nft_transfer(receiver_id.clone(), token_id.clone(), approval_id, None);

        // Payout calculation
        let previous_owner_id = previous_token.owner_id;
        let mut total_perpetual = 0;
        let payout = if let Some(balance) = balance {
            let balance_u128: u128 = u128::from(balance);
            let mut payout: Payout = Payout { payout: HashMap::new() };

            let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
            let token_series_id = token_id_iter.next().unwrap().parse().unwrap();
            let royalty = self.token_series_by_id.get(&token_series_id).expect("no type").royalty;

            assert!(royalty.len() as u32 <= max_len_payout.unwrap(), "Market cannot payout to that many receivers");
            for (k, v) in royalty.iter() {
                let key = k.clone();
                if key != previous_owner_id {
                    payout.payout.insert(key, royalty_to_payout(*v, balance_u128));
                    total_perpetual += *v;
                }
            }

            assert!(
                total_perpetual <= 10000,
                "Total payout overflow"
            );

            payout.payout.insert(previous_owner_id.clone(), royalty_to_payout(10000 - total_perpetual, balance_u128));
            Some(payout)
        } else {
            None
        };

        payout
    }



}


/* codigo costumizado */
fn royalty_to_payout(a: u32, b: Balance) -> U128 {
    U128(a as u128 * b / 10_000u128)
}


near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[near_bindgen]
impl NonFungibleTokenResolver for Contract {
    #[private]
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool {
        let resp: bool = self.tokens.nft_resolve_transfer(
            previous_owner_id.clone(),
            receiver_id.clone(),
            token_id.clone(),
            approved_account_ids,
        );

        // if not successful, return nft back to original owner
        if !resp {
            NearEvent::log_nft_transfer(
                receiver_id.to_string(),
                previous_owner_id.to_string(),
                vec![token_id],
                None,
                None,
            );
        }

        resp
    }
}


fn refund_deposit(storage_used: u64, extra_spend: Balance) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit() - extra_spend;

    assert!(
        required_cost <= attached_deposit,
        "Must attach {} yoctoNEAR to cover storage",
        required_cost,
    );

    let refund = attached_deposit - required_cost;
    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}




/*----------- test --------------*/
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use std::collections::HashMap;

    use super::*;

    const MINT_STORAGE_COST: u128 = 5870000000000000000000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    fn sample_token_metadata() -> TokenMetadata {
        TokenMetadata {
            title: Some("Olympus Mons".into()),
            description: Some("The tallest mountain in the charted solar system".into()),
            media: None,
            media_hash: None,
            copies: Some(1u64),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.nft_token("1".to_string()), None);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_mint() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());

        let token_id = "0".to_string();
        let token = contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());
        assert_eq!(token.token_id, token_id);
        assert_eq!(token.owner_id.to_string(), accounts(0).to_string());
        assert_eq!(token.metadata.unwrap(), sample_token_metadata());
        assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_transfer(accounts(1), token_id.clone(), None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        if let Some(token) = contract.nft_token(token_id.clone()) {
            assert_eq!(token.token_id, token_id);
            assert_eq!(token.owner_id.to_string(), accounts(1).to_string());
            assert_eq!(token.metadata.unwrap(), sample_token_metadata());
            assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
        } else {
            panic!("token not correctly created, or not found by nft_token");
        }
    }

    #[test]
    fn test_approve() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }

    #[test]
    fn test_revoke() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke(token_id.clone(), accounts(1));
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), None));
    }

    #[test]
    fn test_revoke_all() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke_all(token_id.clone());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }
}
use hex;
use near_contract_standards::non_fungible_token::Token;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, UnorderedMap},
    env,
    json_types::{Base58PublicKey, U128},
    near_bindgen, AccountId, Balance, Gas, Promise, PublicKey, StorageUsage,
};

use near_sdk::json_types::U128;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const MINT_FEE: u128 = 1_000_000_000_000_000_000_000_000;
const GUEST_MINT_LIMIT: u8 = 3;

pub type TokenId = String;
pub struct Core {
    pub token: LookupMap<TokenId, Token>,
    owner_id: AccountId,
    token_storage_usage: StorageUsage,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct TokenData {
    pub owner_id: AccountId,
    pub metatdata: String,
    pub price: U128,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NonFungibleTokenBasic {
    pub pubkey_minted: UnorderedMap<PublicKey, u8>,
    pub token_to_data: UnorderedMap<TokenId, TokenData>,
    pub account_to_proceeds: UnorderedMap<AccountId, Balance>,
    pub owner_id: AccountId,
    pub token_id: TokenId,
}

impl Default for NonFungibleTokenBasic {
    fn default() -> Self {
        panic!("NFT should be initialized before usage")
    }
}

impl NonFungibleTokenBasic {
    // init
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(
            env::is_valid_account_id(owner_id.as_bytes()),
            "Owner's account ID is invalid"
        );
        assert!(!env::state_exists(), "Already initialized");
        Self {
            pubkey_minted: UnorderedMap::new(b"pubkey_minted".to_vec()),
            token_to_data: UnorderedMap::new(b"token_to_data".to_vec()),
            account_to_proceeds: UnorderedMap::new(b"account_to_proceeds".to_vec()),
            owner_id,
            token_id: String::new("0"),
        }
    }

    // view methodss
    pub fn get_token_data(&self, token_id: TokenId) -> TokenData {
        match self.token_to_data.get(&token_id) {
            Some(token_data) => token_data,
            None => env::panic(b"No token exists"),
        }
    }

    pub fn get_num_tokens(&self) -> TokenId {
        self.token_id.clone()
    }

    pub fn get_pubkey_minted(&self, pubkey: Base58PublicKey) -> u8 {
        self.pubkey_minted.get(&pubkey.into()).unwrap_or(0)
    }

    pub fn get_proceeds(&self, owner_id: AccountId) -> U128 {
        self.account_to_proceeds.get(&owner_id).unwrap_or(0).into()
    }

    // modifiers
    fn only_owner(&mut self, account_id: AccountId) {
        let singer = env::signer_account_id();
        if singer != account_id {
            let implicit_account_id: AccountId = hex::encode(&env::signer_account_pk()[1..]);
            if implicit_account_id != account_id {
                env::panic(b"Attempt to call transfer on token belonging to another account.");
            }
        }
    }

    fn only_contract_owner(&mut self) {
        assert_eq!(
            env::signer_account_id(),
            self.owner_id,
            "only_contract_owner: Only contract onwer can call method."
        );
    }

    // NFT method
    pub fn nft_transfer(&mut self, receiver_id: AccountId, token_id: TokenId) {
        let mut token_data = self.get_token_data(token_id.clone());
        self.only_owner(token_data.owner_id.clone());
        token_data.owner_id = receiver_id;
        self.token_to_data.insert(&token_id, &token_data);
    }

    pub fn set_price(&mut self, token_id: TokenId, amount: U128) {
        let mut token_data = self.get_token_data(token_id.clone());
        self.only_owner(token_data.owner_id.clone());
        token_data.price = amount.into();
        self.token_to_data.insert(&token_id, &token_data);
    }

    // token purchase - proceeds of sale in escrow for token owner
    pub fn purchase(&mut self, new_owner_id: AccountId, token_id: TokenId) {
        let mut token_data = self.get_token_data(token_id.clone());
        let price = token_data.price.into();
        assert!(price > 0, "not for sale");
        let deposit = env::attached_deposit();
        assert!(deposit == price, "deposit != price");
        // update proceeds balance
        let mut balance = self
            .account_to_proceeds
            .get(&token_data.owner_id)
            .unwrap_or(0);
        balance += deposit;
        self.account_to_proceeds
            .insert(&token_data.owner_id, &balance);

        // transfer ownership
        token_data.owner_id = new_owner_id;
        token_data.price = U128(0);
        self.token_to_data.insert(&token_id, &token_data);
    }
}

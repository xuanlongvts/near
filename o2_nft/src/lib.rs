// #![deny(warnings)]
use hex;
use near_contract_standards::non_fungible_token::Token;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, UnorderedMap},
    env,
    json_types::{Base58PublicKey, U128},
    near_bindgen, AccountId, Balance, Gas, Promise, PublicKey, StorageUsage,
};

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

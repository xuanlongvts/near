use near_sdk::{
	borsh::{self, BorshDeserialize, BorshSerialize},
	collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet},
	env, ext_contract,
	json_types::{Base64VecU8, U128},
	log, near_bindgen,
	serde::{Deserialize, Serialize},
	AccountId, Balance, CryptoHash, Gas, PanicOnDefault, Promise, PromiseOrValue, PromiseResult,
};
use std::collections::HashMap;

pub use crate::approval::*;
use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::royalty::*;

mod approval;
mod enumeration;
mod internal;
mod metadata;
mod mint;
mod nft_core;
mod royalty;

#[derive(BorshSerialize)]
pub enum StorageKey {
	TokensPerOwner,
	TokenPerOwnerInner { account_id_hash: CryptoHash },
	TokensById,
	TokenMetadataById,
	NFTContractMetadata,
	TokensPerType,
	TokensPerTypeInner { token_type_hash: CryptoHash },
	TokenTypesLocked,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
	// contract owner
	pub owner_id: AccountId,
	// keeps track of all the token IDs for a given account
	pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,
	// keeps track of the token struct for a given ID
	pub tokens_by_id: LookupMap<TokenId, Token>,
	// keeps track of the token metadata for a given ID
	pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,
	// keeps track of the metadata for the contract
	pub metadata: LazyOption<NFTContractMetadata>,
}

#[near_bindgen]
impl Contract {
	#[init]
	pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
		Self {
			owner_id,
			tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
			tokens_by_id: LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
			token_metadata_by_id: UnorderedMap::new(
				StorageKey::TokenMetadataById.try_to_vec().unwrap(),
			),
			metadata: LazyOption::new(
				StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
				Some(&metadata),
			),
		}
	}

	#[init]
	pub fn new_default_meta(owner_id: AccountId) -> Self {
		Self::new(
			owner_id,
			NFTContractMetadata {
				spec: "nft-1.0.0".to_string(),
				name: "NFT Tutorial Contract".to_string(),
				symbol: "GOTEAM".to_string(),
				icon: None,
				base_uri: None,
				reference: None,
				reference_hash: None,
			},
		)
	}
}

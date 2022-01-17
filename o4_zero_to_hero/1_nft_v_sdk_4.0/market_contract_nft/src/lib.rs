use near_sdk::{
    assert_one_yocto,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, UnorderedMap, UnorderedSet},
    env,
    env::STORAGE_PRICE_PER_BYTE,
    ext_contract,
    json_types::{U128, U64},
    near_bindgen, promise_result_as_success,
    serde::{Deserialize, Serialize},
    AccountId, Balance, BorshStorageKey, CryptoHash, Gas, PanicOnDefault, Promise,
};
use std::collections::HashMap;

use crate::consts_statics_types::*;
use crate::external::*;
use crate::internal::*;
use crate::sale::*;
mod consts_statics_types;
mod external;
mod internal;
mod nft_callbacks;
mod sale;
mod sale_views;

// Defines the payout type we'll be parsing from the NFT contract as a part of the royalty standard.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    /*
        to keep track of the sales, we map the TypeContractAndTokenId to a Sale.
        the TypeContractAndTokenId is the unique identifier for every sale. It is made
        up of the `contract ID + DELIMITER + token ID`
    */
    pub sales: UnorderedMap<TypeContractAndTokenId, StructSale>,
    // keep track of all the Sale IDs for every account ID
    pub by_owner_id: LookupMap<AccountId, UnorderedSet<TypeContractAndTokenId>>,
    // keep track of all the token IDs for sale for a given contract
    pub by_nft_contract_id: LookupMap<AccountId, UnorderedSet<TypeTokenId>>,
    // keep track of the storage that accounts have payed
    pub storage_deposits: LookupMap<AccountId, Balance>,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum EnumStorageKey {
    Sales,
    ByOwnerId,
    ByOwnerIdInner { account_id_hash: CryptoHash },
    ByNFTContractId,
    ByNFTContractIdInner { account_id_hash: CryptoHash },
    ByNFTTokenType,
    ByNFTTokenTypeInner { token_type_hash: CryptoHash },
    FTTokenIds,
    StorageDeposits,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            sales: UnorderedMap::new(EnumStorageKey::Sales),
            by_owner_id: LookupMap::new(EnumStorageKey::ByOwnerId),
            by_nft_contract_id: LookupMap::new(EnumStorageKey::ByNFTContractId),
            storage_deposits: LookupMap::new(EnumStorageKey::StorageDeposits),
        }
    }

    // Allows users to deposit storage. This is to cover the cost of storing sale objects on the contract
    // Optional account ID is to users can pay for storage for other people.
    pub fn storage_deposits(&mut self, account_id: Option<AccountId>) {
        // get the account ID to pay for storage for
        let storage_account_id = account_id
            // convert the valid account ID into an account ID
            .map(|item| item.into())
            .unwrap_or_else(env::predecessor_account_id);

        let deposit = env::attached_deposit();
        assert!(
            deposit >= CONST_STORAGE_PER_SALE,
            "Requires minimum deposit of {}",
            deposit
        );
        let mut balance: u128 = self.storage_deposits.get(&storage_account_id).unwrap_or(0);
        balance += deposit;
        self.storage_deposits.insert(&storage_account_id, &balance);
    }

    // Allows users to withdraw any excess storage that they're not using. Say Bob pays 0.01N for 1 sale
    // Alice then buys Bob's token. This means bob has paid 0.01N for a sale that's no longer on the marketplace
    // Bob could then withdraw this 0.01N back into his account.
    #[payable]
    pub fn storage_withdraw(&mut self) {
        // make sure the user attaches exactly 1 yoctoNEAR for security purposes.
        // this will redirect them to the NEAR wallet (or requires a full access key).
        assert_one_yocto();

        let owner_id = env::predecessor_account_id();
        let mut amount = self.storage_deposits.remove(&owner_id).unwrap_or(0);
        // how many sales is that user taking up currently. This returns a set
        let sales = self.by_owner_id.get(&owner_id);
        // get the length of that set.
        let len = sales.map(|item| item.len()).unwrap_or_default();
        // how much NEAR is being used up for all the current sales on the account
        let diff = u128::from(len) * CONST_STORAGE_PER_SALE;

        // the excess to withdraw is the total storage paid - storage being used up.
        amount -= diff;

        // if that excess to withdraw is > 0, we transfer the amount to the user.
        if amount > 0 {
            Promise::new(owner_id.clone()).transfer(amount);
        }
        // we need to add back the storage being used up into the map if it's greater than 0.
        // this is so that if the user had 500 sales on the market, we insert that value here so
        // if those sales get taken down, the user can then go and withdraw 500 sales worth of storage.
        if diff > 0 {
            self.storage_deposits.insert(&owner_id, &diff);
        }
    }

    // Views return the minimum storage for 1 sale
    pub fn storage_minimum_balance(&self) -> U128 {
        U128(CONST_STORAGE_PER_SALE)
    }

    // Return how much storage an account has paid for
    pub fn storage_balance_of(&self, account_id: AccountId) -> U128 {
        U128(self.storage_deposits.get(&account_id).unwrap_or(0))
    }
}

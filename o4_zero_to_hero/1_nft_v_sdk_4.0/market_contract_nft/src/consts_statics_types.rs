use near_sdk::{env::STORAGE_PRICE_PER_BYTE, json_types::U128, AccountId, Balance, Gas};

// GAS constants to attach to calls
pub const CONST_GAS_FOR_ROYALTIES: Gas = Gas(115_000_000_000_000);
pub const CONST_GAS_FOR_NFT_TRANSFER: Gas = Gas(15_000_000_000_000);

// Constant used to attach 0 NEAR to a call
pub const CONST_NO_DEPOSIT: Balance = 0;

// The minimum storage to have a sale on the contract.
pub const CONST_STORAGE_PER_SALE: u128 = 1000 * STORAGE_PRICE_PER_BYTE;

// Every sale will have a unique ID which is `CONTRACT + DELIMITER + TOKEN_ID`
pub static STATIC_DELIMITER: &str = ".";

// Creating custom types to use within the contract. This makes things more readable.
pub type TypeSalePriceInYoctoNear = U128;
pub type TypeTokenId = String;
pub type TypeFungibleTokenId = AccountId;
pub type TypeContractAndTokenId = String;

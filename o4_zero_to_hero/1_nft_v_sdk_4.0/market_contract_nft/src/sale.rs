use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StructSale {
	pub owner_id: AccountId,
	pub approval_id: u64,
	pub nft_contract_id: String,
	pub token_id: TypeTokenId,
	pub sale_conditions: TypeSalePriceInYoctoNear,
}

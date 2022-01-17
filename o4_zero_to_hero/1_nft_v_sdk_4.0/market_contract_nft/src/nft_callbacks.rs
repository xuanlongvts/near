use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleArgs {
	pub sale_conditions: TypeSalePriceInYoctoNear,
}

/*
	trait that will be used as the callback from the NFT contract. When nft_approve is
	called, it will fire a cross contract call to this marketplace and this is the function
	that is invoked.
*/
trait NonFungibleTokenApprovalsReceiver {
	fn nft_on_approve(
		&mut self,
		token_id: TypeTokenId,
		owner_id: AccountId,
		approval_id: u64,
		msg: String,
	);
}

#[near_bindgen]
impl NonFungibleTokenApprovalsReceiver for Contract {
	fn nft_on_approve(
		&mut self,
		token_id: TypeTokenId,
		owner_id: AccountId,
		approval_id: u64,
		msg: String,
	) {
		let nft_contract_id = env::predecessor_account_id();

		let signer_id = env::signer_account_id();

		// Make sure that the signer isn't the predecessor. This is so that we're sure this was called via a cross-contract call
		assert_ne!(
			nft_contract_id, signer_id,
			"nft_on_approve should only be called via cross-contract call"
		);

		// Make sure the owner ID is the signer.
		assert_eq!(owner_id, signer_id, "owner_id should be signer_id");

		// We need to enforce that the user has enough storage for 1 EXTRA sale.

		// Get the storage for a sale. dot 0 converts from U128 to u128
		let storage_amount = self.storage_minimum_balance().0;

		// Get the total storage paid by the owner
		let owner_paid_storage = self.storage_deposits.get(&signer_id).unwrap_or(0);

		// Get the storage required which is simply the storage for the number of sales they have + 1
		let signer_storage_required =
			(self.get_supply_by_owner_id(signer_id).0 + 1) as u128 * storage_amount;

		// Make sure that the total paid is >= the required storage
		assert!(
			owner_paid_storage >= signer_storage_required,
			"Insufficient storage paid: {}, for {} sales at {} rate per sale",
			owner_paid_storage,
			signer_storage_required / CONST_STORAGE_PER_SALE,
			CONST_STORAGE_PER_SALE
		);

		// if all these checks pass we can create the sale conditions object.
		let SaleArgs { sale_conditions } =
			near_sdk::serde_json::from_str(&msg).expect("No valid SaleArgs");

		// Create the unique sale ID which is the contract + DELIMITER + token ID
		// The sale conditions come from the msg field. The market assumes that the user passed in a proper msg. If they didn't, it panics.
		let contract_and_token_id = format!("{}{}{}", nft_contract_id, STATIC_DELIMITER, token_id);

		self.sales.insert(
			&contract_and_token_id,
			&StructSale {
				owner_id: owner_id.clone(),
				approval_id,
				nft_contract_id: nft_contract_id.to_string(),
				token_id: token_id.clone(),
				sale_conditions,
			},
		);

		// Extra functionality that populates collections necessary for the view calls
		// Get the sales by owner ID for the given owner. If there are none, we create a new empty set
		let mut by_owner_id = self.by_owner_id.get(&owner_id).unwrap_or_else(|| {
			UnorderedSet::new(
				EnumStorageKey::ByOwnerIdInner {
					account_id_hash: hash_account_id(&owner_id),
				}
				.try_to_vec()
				.unwrap(),
			)
		});
		// insert the unique sale ID into the set
		by_owner_id.insert(&contract_and_token_id);
		// insert that set back into the collection for the owner
		self.by_owner_id.insert(&owner_id, &by_owner_id);

		// Get the token IDs for the given nft contract ID. If there are none, we create a new empty set
		let mut by_nft_contract_id = self
			.by_nft_contract_id
			.get(&nft_contract_id)
			.unwrap_or_else(|| {
				UnorderedSet::new(
					EnumStorageKey::ByNFTContractIdInner {
						account_id_hash: hash_account_id(&nft_contract_id),
					}
					.try_to_vec()
					.unwrap(),
				)
			});

		// insert the token ID into the set
		by_nft_contract_id.insert(&token_id);
		self.by_nft_contract_id
			.insert(&nft_contract_id, &by_nft_contract_id);
	}
}

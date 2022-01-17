use crate::*;

pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
	let mut hash = CryptoHash::default();
	hash.copy_from_slice(&env::sha256(account_id.as_bytes()));

	hash
}

impl Contract {
	// internal method for removing a sale from the market. This returns the previously removed sale object
	pub(crate) fn internal_remove_sale(
		&mut self,
		nft_contract_id: AccountId,
		token_id: TypeTokenId,
	) -> StructSale {
		let contract_and_token_id = format!("{}{}{}", &nft_contract_id, STATIC_DELIMITER, token_id);

		// Get the sale object by removing the unique sale ID. If there was no sale, panic
		let sale = self.sales.remove(&contract_and_token_id).expect("No sale");

		// Get the set of sales for the sale's owner. If there's no sale, panic.
		let mut by_owner_id = self
			.by_owner_id
			.get(&sale.owner_id)
			.expect("No sale by_owner_id");

		by_owner_id.remove(&contract_and_token_id);

		if by_owner_id.is_empty() {
			self.by_owner_id.remove(&sale.owner_id);
		} else {
			self.by_owner_id.insert(&sale.owner_id, &by_owner_id);
		}

		let mut by_nft_contract_id = self
			.by_nft_contract_id
			.get(&nft_contract_id)
			.expect("No sale by nft_contract_id");
		// Remove the token ID from the set
		by_nft_contract_id.remove(&token_id);

		if by_nft_contract_id.is_empty() {
			self.by_nft_contract_id.remove(&nft_contract_id);
		} else {
			self.by_nft_contract_id
				.insert(&nft_contract_id, &by_nft_contract_id);
		}

		sale
	}
}

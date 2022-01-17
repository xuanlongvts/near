use crate::*;

#[near_bindgen]
impl Contract {
	// Returns the number of sales the marketplace has up (as a string)
	pub fn get_supply_sales(&self) -> U64 {
		U64(self.sales.len())
	}

	// Returns the number of sales for a given account (result is a string)
	pub fn get_supply_by_owner_id(&self, account_id: AccountId) -> U64 {
		let by_owner_id = self.by_owner_id.get(&account_id);

		if let Some(by_owner_id) = by_owner_id {
			U64(by_owner_id.len())
		} else {
			U64(0)
		}
	}

	// Returns paginated sale objects for a given account. (result is a vector of sales/)
	pub fn get_sales_by_owner_id(
		&self,
		account_id: AccountId,
		from_index: Option<U128>,
		limit: Option<u64>,
	) -> Vec<StructSale> {
		let by_owner_id = self.by_owner_id.get(&account_id);

		let sales = if let Some(by_owner_id) = by_owner_id {
			by_owner_id
		} else {
			return vec![];
		};
		// We'll convert the UnorderedSet into a vector of strings
		let keys = sales.as_vector();

		// Where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
		let start = u128::from(from_index.unwrap_or(U128(0)));

		keys.iter()
			.skip(start as usize)
			.take(limit.unwrap_or(0) as usize)
			.map(|token_id| self.sales.get(&token_id).unwrap())
			.collect()
	}

	pub fn get_supply_by_nft_contract_id(&self, nft_contract_id: AccountId) -> U64 {
		let by_nft_contract_id = self.by_nft_contract_id.get(&nft_contract_id);

		if let Some(by_nft_contract_id) = by_nft_contract_id {
			U64(by_nft_contract_id.len())
		} else {
			U64(0)
		}
	}

	pub fn get_sales_by_nft_contract_id(
		&self,
		nft_contract_id: AccountId,
		from_index: Option<U128>,
		limit: Option<u64>,
	) -> Vec<StructSale> {
		let by_nft_contract_id = self.by_nft_contract_id.get(&nft_contract_id);

		let sales = if let Some(by_nft_contract_id) = by_nft_contract_id {
			by_nft_contract_id
		} else {
			return vec![];
		};

		// We'll convert the UnorderedSet into a vector of strings
		let keys = sales.as_vector();

		//where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
		let start = u128::from(from_index.unwrap_or(U128(0)));
		//iterate through the keys vector
		keys.iter()
			//skip to the index we specified in the start variable
			.skip(start as usize)
			//take the first "limit" elements in the vector. If we didn't specify a limit, use 0
			.take(limit.unwrap_or(0) as usize)
			//we'll map the token IDs which are strings into Sale objects by passing in the unique sale ID (contract + DELIMITER + token ID)
			.map(|token_id| {
				self.sales
					.get(&format!(
						"{}{}{}",
						nft_contract_id, STATIC_DELIMITER, token_id
					))
					.unwrap()
			})
			//since we turned the keys into an iterator, we need to turn it back into a vector to return
			.collect()
	}

	pub fn get_sales(&self, nft_contract_token: TypeContractAndTokenId) -> Option<StructSale> {
		self.sales.get(&nft_contract_token)
	}
}

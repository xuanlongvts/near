use crate::*;

#[near_bindgen]
impl Contract {
	pub fn nft_tokens_for_owner(
		&self,
		account_id: AccountId,
		from_index: Option<U128>,
		limit: Option<u64>,
	) -> Vec<JsonToken> {
		// get the set of tokens for the passed in owner
		let tokens_for_owner_set = self.tokens_per_owner.get(&account_id);

		// if there is some set of tokens, we'll set the tokens variable equal to that set
		let tokens = if let Some(tokens_for_owner_set_exists) = tokens_for_owner_set {
			tokens_for_owner_set_exists
		} else {
			return vec![];
		};

		// We'll convert the UnorderedSet into a vector of strings
		let keys = tokens.as_vector();

		// Where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
		let start = u128::from(from_index.unwrap_or(U128(0)));

		// iterate through the keys vector
		keys.iter()
			// skip to the index we specified in the start variable
			.skip(start as usize)
			// take the first "limit" elements in the vector. If we didn't specify a limit, use 0
			.take(limit.unwrap_or(0) as usize)
			// we'll map the token IDs which are strings into Json Tokens
			.map(|token_id| self.nft_token(token_id.clone()).unwrap())
			// since we turned the keys into an iterator, we need to turn it back into a vector to return
			.collect()
	}
}

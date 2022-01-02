use crate::*;

// const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(10_000_000_000_000);
// const GAS_FOR_NFT_TRANSFER_CALL: Gas = Gas(5_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER.0);
// const NO_DEPOSITE: Balance = 0;

pub trait NonFungibleTokenCore {
	fn nft_token(&self, token_id: TokenId) -> Option<JsonToken>;
}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {
	// get the information for a specific token ID
	fn nft_token(&self, token_id: TokenId) -> Option<JsonToken> {
		if let Some(token) = self.tokens_by_id.get(&token_id) {
			let metadata = self.token_metadata_by_id.get(&token_id).unwrap();
			return Some(JsonToken {
				token_id,
				owner_id: token.owner_id,
				metadata,
			});
		}

		None
	}
}

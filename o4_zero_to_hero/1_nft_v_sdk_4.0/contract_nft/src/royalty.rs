use crate::*;

pub trait RoyaltyNonFungibleTokenCore {
	// calculates the payout for a token given the passed in balance. This is a view method
	fn nft_payout(&self, token_id: TokenId, balance: Balance, max_len_payout: u32) -> Payout;

	fn nft_transfer_payout(
		&mut self,
		receiver_id: AccountId,
		token_id: TokenId,
		approval_id: u64,
		memo: String,
		balance: Balance,
		max_len_payout: u32,
	) -> Payout;
}

#[near_bindgen]
impl RoyaltyNonFungibleTokenCore for Contract {
	fn nft_payout(&self, token_id: TokenId, balance: Balance, max_len_payout: u32) -> Payout {
		let token = self.tokens_by_id.get(&token_id).expect("No token");
		let owner_id = token.owner_id;

		//keep track of the total perpetual royalties
		let mut total_perpetual = 0;

		let balance_u128 = u128::from(balance);

		let mut payout_object = Payout {
			payout: HashMap::new(),
		};

		let royalty = token.royalty;

		// Make sure we're not paying out to too many people (GAS limits this)
		assert!(
			royalty.len() as u32 <= max_len_payout,
			"Market can not payout to that many receivers"
		);

		for (k, v) in royalty.iter() {
			let key = k.clone();
			if key != owner_id {
				payout_object
					.payout
					.insert(key, royalty_to_payout(*v, balance_u128));
				total_perpetual += *v;
			}
		}

		payout_object.payout.insert(
			owner_id,
			royalty_to_payout(TEN_THOUSANDS - total_perpetual, balance_u128),
		);

		payout_object
	}

	// Transfers the token to the receiver ID and returns the payout object that should be payed given the passed in balance.
	#[payable]
	fn nft_transfer_payout(
		&mut self,
		receiver_id: AccountId,
		token_id: TokenId,
		approval_id: u64,
		memo: String,
		balance: Balance,
		max_len_payout: u32,
	) -> Payout {
		assert_one_yocto();
		// get the sender ID
		let sender_id = env::predecessor_account_id();

		let previous_token = self.internal_transfer(
			&sender_id,
			&receiver_id,
			&token_id,
			Some(approval_id),
			Some(memo),
		);

		// refund the previous token owner for the storage used up by the previous approved account IDs
		refund_approved_account_ids(
			previous_token.owner_id.clone(),
			&previous_token.approved_account_ids,
		);

		let owner_id = previous_token.owner_id;
		let mut total_perpetual = 0;
		let balance_u128 = u128::from(balance);
		let mut payout_object = Payout {
			payout: HashMap::new(),
		};

		// get the royalty object from token
		let royalty = previous_token.royalty;

		// make sure we're not paying out to too many people (GAS limits this)
		assert!(
			royalty.len() as u32 <= max_len_payout,
			"Martket can not payout to that many receivers"
		);

		for (k, v) in royalty.iter() {
			let key = k.clone();
			if key != owner_id {
				payout_object
					.payout
					.insert(key, royalty_to_payout(*v, balance_u128));
				total_perpetual += *v;
			}
		}

		payout_object.payout.insert(
			owner_id,
			royalty_to_payout(TEN_THOUSANDS - total_perpetual, balance_u128),
		);

		payout_object
	}
}

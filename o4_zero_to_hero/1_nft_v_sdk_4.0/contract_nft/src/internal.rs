use crate::*;

pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
	let mut hash = CryptoHash::default();
	hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
	hash
}

// used to make sure the user attached exactly 1 yoctoNEAR
pub(crate) fn assert_one_yocto() {
	assert_eq!(
		env::attached_deposit(),
		1,
		"Requires attached deposit of exactly 1 yoctoNEAR"
	);
}

// Refund the initial deposit based on the amount of storage that was used up
pub(crate) fn refund_deposit(storage_used: u64) {
	// get how much it would cost to store the information
	let required_cost = env::storage_byte_cost() * Balance::from(storage_used);

	// get the attached deposit
	let attached_deposit = env::attached_deposit();

	// make sure that the attached deposit is greater than or equal to the required cost
	assert!(
		required_cost <= attached_deposit,
		"Must attach {} yoctoNEAR to cover storage",
		required_cost
	);

	// get the refund amount from the attached deposit - required cost
	let refund = attached_deposit - required_cost;
	// if the refund is greater than 1 yocto NEAR, we refund the predecessor that amount
	if refund > 1 {
		Promise::new(env::predecessor_account_id()).transfer(refund);
	}
}

impl Contract {
	pub(crate) fn internal_add_token_to_owner(
		&mut self,
		account_id: &AccountId,
		token_id: &TokenId,
	) {
		//get the set of tokens for the given account
		let mut tokens_set = self.tokens_per_owner.get(account_id).unwrap_or_else(|| {
			// if the account doesn't have any tokens, we create a new unordered set
			UnorderedSet::new(
				StorageKey::TokenPerOwnerInner {
					account_id_hash: hash_account_id(&account_id),
				}
				.try_to_vec()
				.unwrap(),
			)
		});
		// We insert the token ID into the set
		tokens_set.insert(token_id);

		// We insert that set for the given account ID.
		self.tokens_per_owner.insert(&account_id, &tokens_set);
	}

	// remove a token from an owner (internal method and can't be called directly via CLI).
	pub(crate) fn internal_remove_token_from_owner(
		&mut self,
		account_id: &AccountId,
		token_id: &TokenId,
	) {
		// we get the set of tokens that the owner has
		let mut tokens_set = self
			.tokens_per_owner
			.get(account_id)
			.expect("Token should be owned by the sender");

		// we remove the the token_id from the set of tokens
		tokens_set.remove(token_id);

		// if the token set is now empty, we remove the owner from the tokens_per_owner collection
		if tokens_set.is_empty() {
			self.tokens_per_owner.remove(account_id);
		} else {
			self.tokens_per_owner.insert(account_id, &tokens_set);
		}
	}

	// transfers the NFT to the receiver_id (internal method and can't be called directly via CLI).
	pub(crate) fn internal_transfer(
		&mut self,
		sender_id: &AccountId,
		receiver_id: &AccountId,
		token_id: &TokenId,
		memo: Option<String>,
	) -> Token {
		let token = self.tokens_by_id.get(token_id).expect("No token");

		// if the sender doesn't equal the owner, we panic
		if sender_id != &token.owner_id {
			env::panic_str("UnAuthorized");
		}

		// we make sure that the sender isn't sending the token to themselves
		assert_ne!(
			&token.owner_id, receiver_id,
			"The token owner and the receiver should be different"
		);

		self.internal_remove_token_from_owner(&token.owner_id, token_id);

		self.internal_add_token_to_owner(receiver_id, token_id);

		// we create a new token struct
		let new_token = Token {
			owner_id: receiver_id.clone(),
		};

		// insert that new token into the tokens_by_id, replacing the old entry
		self.tokens_by_id.insert(token_id, &new_token);

		// if there was some memo attached, we log it.
		if let Some(memo_content) = memo {
			env::log_str(&format!("Memo: {}", memo_content).to_string());
		}

		// return the preivous token object that was transferred.
		token
	}
}

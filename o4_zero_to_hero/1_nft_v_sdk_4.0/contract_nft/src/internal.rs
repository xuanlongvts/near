use crate::*;

pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
	let mut hash = CryptoHash::default();
	hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
	hash
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
}

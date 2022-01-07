use crate::*;

const GAS_FOR_NFT_APPROVE: Gas = Gas(10_000_000_000_000);
const NO_DEPOSIT: Balance = 0;

pub trait ApprovalNonFungibleTokenCore {
	fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>);

	fn nft_is_approved(
		&self,
		token_id: TokenId,
		approved_account_ids: AccountId,
		approval_id: Option<u64>,
	) -> bool;

	fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId);

	fn nft_revoke_all(&mut self, token_id: TokenId);
}

#[ext_contract(ext_non_fungible_approval_receiver)]
trait NonFungibleTokenApprovalsReceiver {
	// cross contract call to an external contract that is initiated during nft_approve
	fn nft_on_approve(
		&mut self,
		token_id: TokenId,
		owner_id: AccountId,
		approval_id: u64,
		msg: String,
	);
}

#[near_bindgen]
impl ApprovalNonFungibleTokenCore for Contract {
	#[payable]
	fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>) {
		assert_at_least_one_yocto();

		let mut token = self.tokens_by_id.get(&token_id).expect("No token");

		assert_eq!(
			&env::predecessor_account_id(),
			&token.owner_id,
			"Predecessor must be the owner."
		);

		// get the next approval ID if we need a new approval
		let approval_id: u64 = token.next_approval_id;

		let is_new_approval = token
			.approved_account_ids
			.insert(account_id.clone(), approval_id)
			.is_none(); // if the key was not present, .is_none() will return true so it is a new approval.

		let storage_used = if is_new_approval {
			bytes_for_approved_account_id(&account_id)
		} else {
			0
		};

		token.next_approval_id += 1;
		self.tokens_by_id.insert(&token_id, &token);
		refund_deposit(storage_used);

		if let Some(mesg) = msg {
			ext_non_fungible_approval_receiver::nft_on_approve(
				token_id,
				token.owner_id,
				approval_id,
				mesg,
				account_id,                               // contract account we're calling
				NO_DEPOSIT,                               // NEAR deposit we attach to the call
				env::prepaid_gas() - GAS_FOR_NFT_APPROVE, // GAS we're attaching
			)
			.as_return(); // Returning this promise
		}
	}

	fn nft_is_approved(
		&self,
		token_id: TokenId,
		approved_account_id: AccountId,
		approval_id: Option<u64>,
	) -> bool {
		// get the token object from the token_id
		let token = self.tokens_by_id.get(&token_id).expect("No token");

		//get the approval number for the passed in account ID
		let approval = token.approved_account_ids.get(&approved_account_id);

		// if there was some approval ID found for the account ID
		if let Some(approval) = approval {
			if let Some(approval_id) = approval_id {
				approval_id == *approval
			} else {
				return true;
			}
		} else {
			false
		}
	}

	//revoke a specific account from transferring the token on your behalf
	#[payable]
	fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId) {
		assert_one_yocto();

		let mut token = self.tokens_by_id.get(&token_id).expect("No token");

		// Get the caller of the function and assert that they are the owner of the token
		let predecessor_account_id = env::predecessor_account_id();
		assert_eq!(&predecessor_account_id, &token.owner_id);

		// if the account ID was in the token's approval, we remove it and the if statement logic executes
		if token.approved_account_ids.remove(&account_id).is_some() {
			// refund the funds released by removing the approved_account_id to the caller of the function
			refund_approved_account_ids_iter(predecessor_account_id, [account_id].iter());

			self.tokens_by_id.insert(&token_id, &token);
		}
	}

	#[payable]
	fn nft_revoke_all(&mut self, token_id: TokenId) {
		assert_one_yocto();

		let mut token = self.tokens_by_id.get(&token_id).expect("No token");

		// Get the caller of the function and assert that they are the owner of the token
		let predecessor_account_id = env::predecessor_account_id();
		assert_eq!(&predecessor_account_id, &token.owner_id);

		if !token.approved_account_ids.is_empty() {
			// refund the approved account IDs to the caller of the function
			refund_approved_account_ids(predecessor_account_id, &token.approved_account_ids);
			token.approved_account_ids.clear();
		}
	}
}

use crate::*;

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(10_000_000_000_000);
const GAS_FOR_NFT_TRANSFER_CALL: Gas = Gas(5_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER.0);
const MIN_GAS_FOR_NFT_TRANSFER_CALL: Gas = Gas(100_000_000_000_000);
const NO_DEPOSIT: Balance = 0;

pub trait CoreNonFungibleToken {
	fn nft_token(&self, token_id: TokenId) -> Option<JsonToken>;

	fn nft_transfer(
		&mut self,
		receiver_id: AccountId,
		token_id: TokenId,
		approval_id: u64,
		memo: Option<String>,
	);

	fn nft_transfer_call(
		&mut self,
		receiver_id: AccountId,
		token_id: TokenId,
		approval_id: u64,
		memo: Option<String>,
		msg: String,
	) -> PromiseOrValue<bool>;
}

#[ext_contract(ext_non_fungible_token_receiver)]
trait NonFungibleTokenReceiver {
	fn nft_on_transfer(
		&mut self,
		sender_id: AccountId,
		previous_owner_id: AccountId,
		token_id: TokenId,
		msg: String,
	) -> Promise;
}

#[ext_contract(ext_self)]
trait NonFungibleTokenResolver {
	fn nft_resolve_transfer(
		&mut self,
		authorized_id: Option<String>,
		owner_id: AccountId,
		receiver_id: AccountId,
		token_id: TokenId,
		approved_account_ids: HashMap<AccountId, u64>,
		memo: Option<String>,
	) -> bool;
}

trait NonFungibleTokenResolver {
	fn nft_resolve_transfer(
		&mut self,
		authorized_id: Option<String>,
		owner_id: AccountId,
		receiver_id: AccountId,
		token_id: TokenId,
		approved_account_ids: HashMap<AccountId, u64>,
		memo: Option<String>,
	) -> bool;
}

#[near_bindgen]
impl CoreNonFungibleToken for Contract {
	// implementation of the nft_transfer method. This transfers the NFT from the current owner to the receiver.
	#[payable]
	fn nft_transfer(
		&mut self,
		receiver_id: AccountId,
		token_id: TokenId,
		approval_id: u64,
		memo: Option<String>,
	) {
		// assert that the user attached exactly 1 yoctoNEAR. This is for security and so that the user will be redirected to the NEAR wallet.
		assert_one_yocto();

		let sender_id = env::predecessor_account_id();

		//call the internal transfer method and get back the previous token so we can refund the approved account IDs
		let previous_token =
			self.internal_transfer(&sender_id, &receiver_id, &token_id, Some(approval_id), memo);

		//we refund the owner for releasing the storage used up by the approved account IDs
		refund_approved_account_ids(
			previous_token.owner_id.clone(),
			&previous_token.approved_account_ids,
		);
	}

	// implementation of the transfer call method. This will transfer the NFT and call a method on the reciver_id contract
	#[payable]
	fn nft_transfer_call(
		&mut self,
		receiver_id: AccountId,
		token_id: TokenId,
		approval_id: u64,
		memo: Option<String>,
		msg: String,
	) -> PromiseOrValue<bool> {
		assert_one_yocto();

		let attached_gas = env::prepaid_gas();

		/*
			make sure that the attached gas is greater than the minimum GAS for NFT transfer call.
			This is to ensure that the cross contract call to nft_on_transfer won't cause a prepaid GAS error.
			If this happens, the event will be logged in internal_transfer but the actual transfer logic will be
			reverted due to the panic. This will result in the databases thinking the NFT belongs to the wrong person.
		*/
		assert!(
			attached_gas >= MIN_GAS_FOR_NFT_TRANSFER_CALL,
			"You cannot attach less than {:?} Gas to nft_transfer_call",
			MIN_GAS_FOR_NFT_TRANSFER_CALL
		);

		// get the sender ID
		let sender_id = env::predecessor_account_id();

		// transfer the token and get the previous token object
		let previous_token = self.internal_transfer(
			&sender_id,
			&receiver_id,
			&token_id,
			Some(approval_id),
			memo.clone(),
		);

		let mut authorized_id = None;
		if sender_id != previous_token.owner_id {
			authorized_id = Some(sender_id.to_string());
		}

		// Initiating receiver's call and the callback
		ext_non_fungible_token_receiver::nft_on_transfer(
			sender_id,
			previous_token.owner_id.clone(),
			token_id.clone(),
			msg,
			receiver_id.clone(), // contract account to make the call to
			NO_DEPOSIT,          // attached deposit
			env::prepaid_gas() - GAS_FOR_NFT_TRANSFER_CALL, //attached GAS
		)
		.then(ext_self::nft_resolve_transfer(
			authorized_id,
			previous_token.owner_id,
			receiver_id,
			token_id,
			previous_token.approved_account_ids,
			memo,
			env::current_account_id(), // contract account to make the call to
			NO_DEPOSIT,                // attached deposit
			GAS_FOR_RESOLVE_TRANSFER,  // GAS attached to the call
		))
		.into()
	}

	// get the information for a specific token ID
	fn nft_token(&self, token_id: TokenId) -> Option<JsonToken> {
		if let Some(token) = self.tokens_by_id.get(&token_id) {
			let metadata = self.token_metadata_by_id.get(&token_id).unwrap();
			return Some(JsonToken {
				token_id,
				owner_id: token.owner_id,
				metadata,
				approved_account_ids: token.approved_account_ids,
				royalty: token.royalty,
			});
		} else {
			None
		}
	}
}

#[near_bindgen]
impl NonFungibleTokenResolver for Contract {
	#[private]
	fn nft_resolve_transfer(
		&mut self,
		authorized_id: Option<String>,
		owner_id: AccountId,
		receiver_id: AccountId,
		token_id: TokenId,
		approved_account_ids: HashMap<AccountId, u64>,
		memo: Option<String>,
	) -> bool {
		// Whether receiver wants to return token back to the sender, based on `nft_on_transfer` call result.
		if let PromiseResult::Successful(value) = env::promise_result(0) {
			// As per the standard, the nft_on_transfer should return whether we should return the token to it's owner or not
			if let Ok(return_token) = near_sdk::serde_json::from_slice::<bool>(&value) {
				// if we need don't need to return the token, we simply return true meaning everything went fine
				if !return_token {
					/*
						since we've already transferred the token and nft_on_transfer returned false, we don't have to
						revert the original transfer and thus we can just return true since nothing went wrong.
					*/
					// we refund the owner for releasing the storage used up by the approved account IDs
					refund_approved_account_ids(owner_id, &approved_account_ids);
					return true;
				}
			}
		}

		// get the token object if there is some token object
		let mut token = if let Some(tok) = self.tokens_by_id.get(&token_id) {
			if tok.owner_id != receiver_id {
				// we refund the owner for releasing the storage used up by the approved account IDs
				refund_approved_account_ids(owner_id, &approved_account_ids);
				// The token is not owner by the receiver anymore. Can't return it.
				return true;
			}
			tok
		} else {
			// we refund the owner for releasing the storage used up by the approved account IDs
			refund_approved_account_ids(owner_id, &approved_account_ids);
			return true;
		};

		// if at the end, we haven't returned true, that means that we should return the token to it's original owner
		log!(
			"Return {} from @{} to @{}",
			token_id,
			receiver_id.clone(),
			owner_id
		);

		// we remove the token from the receiver
		self.internal_remove_token_from_owner(&receiver_id.clone(), &token_id);

		//we add the token to the original owner
		self.internal_add_token_to_owner(&owner_id, &token_id);

		// we change the token struct's owner to be the original owner
		token.owner_id = owner_id.clone();

		// We refund the receiver any approved account IDs that they may have set on the token
		refund_approved_account_ids(receiver_id.clone(), &token.approved_account_ids);

		// Reset the approved account IDs to what they were before the transfer
		token.approved_account_ids = approved_account_ids;

		let nft_transfer_log: EventLog = EventLog {
			standard: NFT_STANDARD_NAME.to_string(),
			version: NFT_METADATA_SPEC.to_string(),
			event: EventLogVariant::NftTransfer(vec![NftTransferLog {
				authorized_id,
				old_owner_id: receiver_id.to_string(),
				new_owner_id: owner_id.to_string(),
				token_ids: vec![token_id.to_string()],
				memo,
			}]),
		};

		env::log_str(&nft_transfer_log.to_string());

		// we inset the token back into the tokens_by_id collection
		self.tokens_by_id.insert(&token_id, &token);

		false
	}
}

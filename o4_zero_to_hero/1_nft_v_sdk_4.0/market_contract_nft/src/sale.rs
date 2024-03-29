use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StructSale {
	pub owner_id: AccountId,
	pub approval_id: u64,
	pub nft_contract_id: String,
	pub token_id: TypeTokenId,
	pub sale_conditions: TypeSalePriceInYoctoNear,
}

#[ext_contract(ext_self)]
trait ExtSelf {
	fn resolve_purchase(&mut self, buyer_id: AccountId, price: U128) -> Promise;
}

#[near_bindgen]
impl Contract {
	// Removes a sale from the market.
	#[payable]
	pub fn remove_sale(&mut self, nft_contract_id: AccountId, token_id: TypeTokenId) {
		assert_one_yocto();
		let sale = self.internal_remove_sale(nft_contract_id, token_id);

		let owner_id = env::predecessor_account_id();

		assert_eq!(owner_id, sale.owner_id, "Must be sale owner");
	}

	// Update the price for sale on the market
	#[payable]
	pub fn update_price(&mut self, nft_contract_id: AccountId, token_id: TypeTokenId, price: U128) {
		assert_one_yocto();

		// crate the unique sale ID from the nft contract and token
		let contract_id: AccountId = nft_contract_id.into();
		let contract_and_token_id = format!("{}{}{}", contract_id, STATIC_DELIMITER, token_id);

		// Get the sale object from the unique sale ID. If there is no token, panic.
		let mut sale = self.sales.get(&contract_and_token_id).expect("No sale");

		// Assert that caller or the function is the sale owner
		assert_eq!(
			env::predecessor_account_id(),
			sale.owner_id,
			"Must be sale owner"
		);
		// Set the sale conditions equal to the passed in price
		sale.sale_conditions = price;

		// Insert the sale back into the map for the unique sale ID
		self.sales.insert(&contract_and_token_id, &sale);
	}

	// Place an offer on a specific sale. The sale will go through as long as your deposit is greater than or equal to the list price
	#[payable]
	pub fn offer(&mut self, nft_contract_id: AccountId, token_id: TypeTokenId) {
		let deposit = env::attached_deposit();
		assert!(deposit > 0, "Attached deposit must be greater than 0");

		// Convert the nft_contract_id from a AccountId to an AccountId
		let contract_id: AccountId = nft_contract_id.into();
		let contract_and_token_id = format!("{}{}{}", contract_id, STATIC_DELIMITER, token_id);

		let sale = self.sales.get(&contract_and_token_id).expect("No sale");

		// Get the buyer ID which is the person who called the function and make sure they're not the owner of the sale
		let buyer_id = env::predecessor_account_id();
		assert_ne!(sale.owner_id, buyer_id, "Can not bid on your own sale.");

		// Get the u128 price of the token (dot 0 converts from U128 to u128)
		let price = sale.sale_conditions.0;

		// Make sure the deposit is greater than the price
		assert!(
			deposit >= price,
			"Attached deposit must be greater than or equal to the current price: {:?}",
			price
		);

		self.process_purchase(contract_id, token_id, U128(deposit), buyer_id);
	}

	// Private function used when a sale is purchased. This will remove the sale, transfer and get the payout from the nft contract, and then distribute royalties
	#[private]
	pub fn process_purchase(
		&mut self,
		nft_contract_id: AccountId,
		token_id: TypeTokenId,
		price: U128,
		buyer_id: AccountId,
	) -> Promise {
		// Get the sale object by removing the sale
		let sale = self.internal_remove_sale(nft_contract_id.clone(), token_id.clone());

		ext_contract::nft_transfer_payout(
			buyer_id.clone(),                  // purchaser (person to transfer the NFT to)
			token_id,                          // token ID to transfer
			sale.approval_id, // market contract's approval ID in order to transfer the token on behalf of the owner
			"payout from martket".to_string(), // memo (to include some context)
			/*
				the price that the token was purchased for. This will be used in conjunction with the royalty percentages
				for the token in order to determine how much money should go to which account.
			*/
			price,
			10, // the maximum amount of accounts the market can payout at once (this is limited by GAS)
			nft_contract_id, // contract to initiate the cross contract call to
			1,  // yoctoNEAR to attach to the call
			CONST_GAS_FOR_NFT_TRANSFER, // GAS to attach to the call
		)
		// after the transfer payout has been initiated, we resolve the promise by calling our own resolve_purchase function.
		// resolve purchase will take the payout object returned from the nft_transfer_payout and actually pay the accounts
		.then(ext_self::resolve_purchase(
			buyer_id,
			price,
			env::current_account_id(), // We are invoking this function on the current contract
			CONST_NO_DEPOSIT,          // don't attach any deposit
			CONST_GAS_FOR_ROYALTIES,   // GAS attached to the call to payout royalties
		))
	}

	/*
		private method used to resolve the promise when calling nft_transfer_payout. This will take the payout object and
		check to see if it's authentic and there's no problems. If everything is fine, it will pay the accounts. If there's a problem,
		it will refund the buyer for the price.
	*/
	#[private]
	pub fn resolve_purchase(&mut self, buyer_id: AccountId, price: U128) -> U128 {
		// checking for payout information returned from the nft_transfer_payout method
		let payout_option = promise_result_as_success().and_then(|value| {
			// if we set the payout_option to None, that means something went wrong and we should refund the buyer
			near_sdk::serde_json::from_slice::<Payout>(&value)
				// converts the result to an optional value
				.ok()
				// returns None if the none. Otherwise executes the following logic
				.and_then(|payout_object| {
					if payout_object.payout.len() > 10 || payout_object.payout.is_empty() {
						env::log_str("Cannot have more than 10 royalties");
						None
					} else {
						// We'll keep track of how much the nft contract wants us to payout. Starting at the full price payed by the buyer
						let mut remainder = price.0;
						// Loop through the payout and subtract the values from the remainder.
						for &value in payout_object.payout.values() {
							// Checked sub checks for overflow or any errors and returns None if there are problems
							remainder = remainder.checked_sub(value.0)?;
						}
						// Check to see if the NFT contract sent back a faulty payout that requires us to pay more or too little.
						// The remainder will be 0 if the payout summed to the total price. The remainder will be 1 if the royalties
						// we something like 3333 + 3333 + 3333.
						if remainder == 0 || remainder == 1 {
							// Set the payout_option to be the payout because nothing went wrong
							Some(payout_object.payout)
						} else {
							// If the remainder was anything but 1 or 0, we return None
							None
						}
					}
				})
		});

		// if the payout option was some payout, we set this payout variable equal to that some payout
		let payout = if let Some(payout_option) = payout_option {
			payout_option
		} else {
			Promise::new(buyer_id).transfer(u128::from(price));
			return price;
		};

		// NEAR payouts
		for (receiver_id, amount) in payout {
			Promise::new(receiver_id).transfer(amount.0);
		}

		price
	}
}

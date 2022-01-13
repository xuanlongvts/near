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
}

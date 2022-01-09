use crate::*;
use std::fmt::{Display, Error, Formatter, Result};

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct NftMinLog {
	pub owner_id: String,
	pub token_id: Vec<TokenId>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub memo: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct NftTransferLog {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub authorized_id: Option<String>,
	pub old_owner_id: String,
	pub new_owner_id: String,
	pub token_ids: Vec<TokenId>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[serde(crate = "near_sdk::serde")]
#[non_exhaustive]
pub enum EventLogVariant {
	NftMint(Vec<NftMinLog>),
	NftTransfer(Vec<NftTransferLog>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct EventLog {
	pub standard: String,
	pub version: String,

	// `flatten` to not have "event": {<EventLogVariant>} in the JSON, just have the contents of {<EventLogVariant>}.
	#[serde(flatten)]
	pub event: EventLogVariant,
}

impl Display for EventLog {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		f.write_fmt(format_args!(
			"EVENT_JSON: {}",
			serde_json::to_string(self).map_err(|_| Error)?
		))
	}
}

// SPDX-License-Identifier: GPL-3.0

use strum::{EnumMessage as _, EnumProperty as _, VariantArray as _};
use strum_macros::{AsRefStr, Display, EnumMessage, EnumProperty, EnumString, VariantArray};
use subxt::{OnlineClient, SubstrateConfig};

use super::find_extrinsic_by_name;

#[derive(
	AsRefStr,
	Clone,
	Debug,
	Display,
	EnumMessage,
	EnumString,
	EnumProperty,
	Eq,
	PartialEq,
	VariantArray,
)]
pub enum Action {
	#[strum(
		serialize = "create",
		message = "create",
		detailed_message = "Create an Asset",
		props(Pallet = "Assets")
	)]
	CreateAsset,
	#[strum(
		serialize = "mint",
		message = "mint",
		detailed_message = "Mint an Asset",
		props(Pallet = "Assets")
	)]
	MintAsset,
	#[strum(
		serialize = "create_nft",
		message = "create",
		detailed_message = "Create an NFT Collection",
		props(Pallet = "Nfts")
	)]
	CreateCollection,
	#[strum(
		serialize = "mint_nft",
		message = "mint",
		detailed_message = "Mint an NFT",
		props(Pallet = "Nfts")
	)]
	MintNFT,
	#[strum(
		serialize = "transfer",
		message = "transfer_allow_death",
		detailed_message = "Transfer Balance",
		props(Pallet = "Balances")
	)]
	Transfer,
}

impl Action {
	/// Get the action's name.
	pub fn action_name(&self) -> &str {
		self.get_message().unwrap_or_default()
	}

	/// Get the description of the action.
	pub fn description(&self) -> &str {
		self.get_detailed_message().unwrap_or_default()
	}

	/// Get the associated pallet for the action.
	pub fn pallet(&self) -> &str {
		self.get_str("Pallet").unwrap_or_default()
	}
}

pub async fn supported_actions(api: &OnlineClient<SubstrateConfig>) -> Vec<Action> {
	let mut actions = Vec::new();
	for action in Action::VARIANTS.iter() {
		if find_extrinsic_by_name(api, action.pallet(), action.action_name()).await.is_ok() {
			actions.push(action.clone());
		}
	}
	actions
}
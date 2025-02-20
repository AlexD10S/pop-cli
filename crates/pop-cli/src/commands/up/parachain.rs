// SPDX-License-Identifier: GPL-3.0

use crate::{
	build::spec::BuildSpecCommand, call::chain::Chain, cli::traits::*,
	common::wallet::submit_extrinsic_with_wallet,
};
use anyhow::{anyhow, Result};
use clap::Args;
use pop_parachains::{
	construct_extrinsic, extract_para_id_from_event, find_dispatchable_by_name,
	parse_chain_metadata, set_up_client, Action, Payload,
};
use std::path::PathBuf;
use url::Url;

const DEFAULT_URL: &str = "wss://paseo.rpc.amforc.com/";
const HELP_HEADER: &str = "Chain deployment options";

#[derive(Args, Clone, Default)]
#[clap(next_help_heading = HELP_HEADER)]
pub struct UpChainCommand {
	/// Path to the chain directory.
	#[clap(skip)]
	pub(crate) path: Option<PathBuf>,
	/// Parachain ID to use. If not specified, a new ID will be reserved.
	#[arg(short, long)]
	pub(crate) id: Option<u32>,
	/// Path to the genesis state file. If not specified, it will be generated.
	#[arg(short = 'G', long = "genesis-state")]
	pub(crate) genesis_state: Option<PathBuf>,
	/// Path to the genesis code file.  If not specified, it will be generated.
	#[arg(short = 'C', long = "genesis-code")]
	pub(crate) genesis_code: Option<PathBuf>,
	/// Websocket endpoint of the relay chain.
	#[arg(long)]
	pub(crate) relay_url: Option<Url>,
}

impl UpChainCommand {
	/// Executes the command.
	pub(crate) async fn execute(self, cli: &mut impl Cli) -> Result<()> {
		cli.intro("Deploy a chain")?;
		let chain_config = match self.prepare_chain_for_registration(cli).await {
			Ok(chain) => chain,
			Err(e) => {
				cli.outro_cancel(format!("{}", e))?;
				return Ok(());
			},
		};
		match chain_config.register_parachain(cli).await {
			Ok(_) => cli.success("Chain deployed successfully")?,
			Err(e) => cli.outro_cancel(format!("{}", e))?,
		}
		Ok(())
	}

	// Prepares the chain for registration by setting up its configuration.
	async fn prepare_chain_for_registration(self, cli: &mut impl Cli) -> Result<UpChain> {
		let chain = self.configure_chain(cli).await?;
		let para_id = self.resolve_parachain_id(&chain, cli).await?;
		let (genesis_state, genesis_code) = self.resolve_genesis_files(para_id, cli).await?;
		Ok(UpChain { id: para_id, genesis_state, genesis_code, chain })
	}

	// Configures the chain by resolving the URL and fetching its metadata.
	async fn configure_chain(&self, cli: &mut impl Cli) -> Result<Chain> {
		// Resolve url.
		let url = match &self.relay_url {
			Some(url) => url.clone(),
			None => {
				// Prompt for url.
				let url: String = cli
					.input("Enter the relay chain node URL to deploy your parachain")
					.default_input(DEFAULT_URL)
					.interact()?;
				Url::parse(&url)?
			},
		};

		// Parse metadata from chain url.
		let client = set_up_client(url.as_str()).await?;
		let pallets = parse_chain_metadata(&client).map_err(|e| {
			anyhow!(format!("Unable to fetch the chain metadata: {}", e.to_string()))
		})?;
		Ok(Chain { url, client, pallets })
	}
	// Resolves the parachain ID, reserving a new one if necessary.
	async fn resolve_parachain_id(&self, chain: &Chain, cli: &mut impl Cli) -> Result<u32> {
		match self.id {
			Some(id) => Ok(id),
			None => {
				cli.info("Reserving a parachain ID...")?;
				return reserve_para_id(&chain, cli).await;
			},
		}
	}
	// Resolves the genesis state and code files, generating them if necessary.
	async fn resolve_genesis_files(
		&self,
		para_id: u32,
		cli: &mut impl Cli,
	) -> Result<(PathBuf, PathBuf)> {
		match (self.genesis_state.clone(), self.genesis_code.clone()) {
			(Some(state), Some(code)) => Ok((state, code)),
			_ => {
				cli.info("Generating the chain spec for your parachain.")?;
				return generate_spec_files(para_id, self.path.clone(), cli).await;
			},
		}
	}
}

// Represents the configuration for deploying a chain.
pub(crate) struct UpChain {
	id: u32,
	genesis_state: PathBuf,
	genesis_code: PathBuf,
	chain: Chain,
}
impl UpChain {
	// Registers a parachain by submitting an extrinsic.
	async fn register_parachain(&self, cli: &mut impl Cli) -> Result<()> {
		cli.info("Registering a parachain ID")?;
		let call_data = self.prepare_register_parachain_call_data()?;
		submit_extrinsic_with_wallet(&self.chain.client, &self.chain.url, call_data, cli).await?;
		Ok(())
	}

	// Prepares and returns the encoded call data for registering a parachain.
	fn prepare_register_parachain_call_data(&self) -> Result<Vec<u8>> {
		let UpChain { id, genesis_code, genesis_state, chain } = self;
		let ex = find_dispatchable_by_name(
			&chain.pallets,
			Action::Register.pallet_name(),
			Action::Register.function_name(),
		)?;
		let state = std::fs::read_to_string(genesis_state)
			.map_err(|err| anyhow!("Failed to read genesis state file: {}", err.to_string()))?;
		let code = std::fs::read_to_string(genesis_code)
			.map_err(|err| anyhow!("Failed to read genesis state file: {}", err.to_string()))?;
		let xt = construct_extrinsic(ex, vec![id.to_string(), state, code])?;
		Ok(xt.encode_call_data(&chain.client.metadata())?)
	}
}

// Reserves a parachain ID by submitting an extrinsic.
async fn reserve_para_id(chain: &Chain, cli: &mut impl Cli) -> Result<u32> {
	let call_data = prepare_reserve_parachain_call_data(chain)?;
	let events = submit_extrinsic_with_wallet(&chain.client, &chain.url, call_data, cli)
		.await
		.map_err(|e| anyhow::anyhow!("Parachain ID reservation failed: {}", e))?;
	let para_id = extract_para_id_from_event(&events).map_err(|_| {
		anyhow::anyhow!("Unable to parse the event. Specify the parachain ID manually with --id.")
	})?;
	cli.success(format!("Successfully reserved parachain ID: {}", para_id))?;
	Ok(para_id)
}

// Prepares and returns the encoded call data for reserving a parachain ID.
fn prepare_reserve_parachain_call_data(chain: &Chain) -> Result<Vec<u8>> {
	let function = find_dispatchable_by_name(
		&chain.pallets,
		Action::Reserve.pallet_name(),
		Action::Reserve.function_name(),
	)?;
	let xt = construct_extrinsic(function, Vec::new())?;
	Ok(xt.encode_call_data(&chain.client.metadata())?)
}

// Generates chain spec files for the parachain.
async fn generate_spec_files(
	id: u32,
	path: Option<PathBuf>,
	cli: &mut impl Cli,
) -> anyhow::Result<(PathBuf, PathBuf)> {
	// Changes the working directory if a path is provided to ensure the build spec process runs in
	// the correct context.
	if let Some(path) = path {
		std::env::set_current_dir(path)?;
	}
	let build_spec = BuildSpecCommand {
		id: Some(id),
		genesis_code: true,
		genesis_state: true,
		..Default::default()
	}
	.configure_build_spec(cli)
	.await?;
	build_spec.generate_genesis_artifacts(cli)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::cli::MockCli;
	use pop_parachains::decode_call_data;
	use std::fs;
	use tempfile::tempdir;
	use url::Url;

	const POLKADOT_NETWORK_URL: &str = "wss://polkadot-rpc.publicnode.com";
	const POP_NETWORK_TESTNET_URL: &str = "wss://rpc1.paseo.popnetwork.xyz";

	#[tokio::test]
	async fn prepare_chain_for_registration_works() -> Result<()> {
		let mut cli = MockCli::new().expect_input(
			"Enter the relay chain node URL to deploy your parachain",
			POLKADOT_NETWORK_URL.into(),
		);
		let (genesis_state, genesis_code) = create_temp_genesis_files()?;
		let chain_config = UpChainCommand {
			id: Some(2000),
			genesis_state: Some(genesis_state.clone()),
			genesis_code: Some(genesis_code.clone()),
			..Default::default()
		}
		.prepare_chain_for_registration(&mut cli)
		.await?;

		assert_eq!(chain_config.id, 2000);
		assert_eq!(chain_config.genesis_code, genesis_code);
		assert_eq!(chain_config.genesis_state, genesis_state);
		assert_eq!(chain_config.chain.url, Url::parse(POLKADOT_NETWORK_URL)?);
		cli.verify()
	}

	#[tokio::test]
	async fn prepare_reserve_parachain_call_data_works() -> Result<()> {
		let mut cli = MockCli::new();
		let chain = UpChainCommand {
			relay_url: Some(Url::parse(POLKADOT_NETWORK_URL)?),
			..Default::default()
		}
		.configure_chain(&mut cli)
		.await?;
		let call_data = prepare_reserve_parachain_call_data(&chain)?;
		assert_eq!(call_data, decode_call_data("0x4605")?);
		Ok(())
	}

	#[tokio::test]
	async fn register_parachain_fails_wrong_chain() -> Result<()> {
		let mut cli = MockCli::new()
			.expect_intro("Deploy a chain")
			.expect_info("Registering a parachain ID")
			.expect_outro_cancel("Failed to find the pallet Registrar");
		let (genesis_state, genesis_code) = create_temp_genesis_files()?;
		UpChainCommand {
			id: Some(2000),
			genesis_state: Some(genesis_state.clone()),
			genesis_code: Some(genesis_code.clone()),
			relay_url: Some(Url::parse(POP_NETWORK_TESTNET_URL)?),
			path: None,
		}
		.execute(&mut cli)
		.await?;

		cli.verify()
	}

	#[tokio::test]
	async fn prepare_register_parachain_call_data_works() -> Result<()> {
		let mut cli = MockCli::new();
		let chain = UpChainCommand {
			relay_url: Some(Url::parse(POLKADOT_NETWORK_URL)?),
			..Default::default()
		}
		.configure_chain(&mut cli)
		.await?;
		// Create a temporary files to act as genesis_state and genesis_code files.
		let temp_dir = tempdir()?;
		let genesis_state_path = temp_dir.path().join("genesis_state");
		let genesis_code_path = temp_dir.path().join("genesis_code.wasm");
		std::fs::write(&genesis_state_path, "0x1234")?;
		std::fs::write(&genesis_code_path, "0x1234")?;

		let call_data = UpChain {
			id: 2000,
			genesis_state: genesis_state_path,
			genesis_code: genesis_code_path,
			chain,
		}
		.prepare_register_parachain_call_data()?;
		assert_eq!(call_data, decode_call_data("0x4600d0070000081234081234")?);
		Ok(())
	}

	// Creates temporary files to act as `genesis_state` and `genesis_code` files.
	fn create_temp_genesis_files() -> Result<(PathBuf, PathBuf)> {
		let temp_dir = tempdir()?; // Create a temporary directory
		let genesis_state_path = temp_dir.path().join("genesis_state");
		let genesis_code_path = temp_dir.path().join("genesis_code.wasm");

		fs::write(&genesis_state_path, "0x1234")?;
		fs::write(&genesis_code_path, "0x1234")?;

		Ok((genesis_state_path, genesis_code_path))
	}
}

// SPDX-License-Identifier: GPL-3.0

use crate::{
	cli::{traits::Cli as _, Cli},
	common::contracts::check_contracts_node_and_prompt,
};
use clap::Args;
use cliclack::{clear_screen, log::warning, outro};
use pop_contracts::{test_e2e_smart_contract, test_smart_contract};
use std::path::PathBuf;
#[cfg(not(test))]
use {std::time::Duration, tokio::time::sleep};

#[derive(Args)]
pub(crate) struct TestContractCommand {
	#[arg(short, long, help = "Path for the contract project [default: current directory]")]
	path: Option<PathBuf>,
	/// [DEPRECATED] Run e2e tests
	#[arg(short, long, value_parser=["e2e-tests"])]
	features: Option<String>,
	/// Run end-to-end tests
	#[arg(short, long)]
	e2e: bool,
	#[arg(short, long, help = "Path to the contracts node to run e2e tests [default: none]")]
	node: Option<PathBuf>,
	/// Automatically source the needed binary required without prompting for confirmation.
	#[clap(short = 'y', long)]
	skip_confirm: bool,
}

impl TestContractCommand {
	/// Executes the command.
	pub(crate) async fn execute(mut self) -> anyhow::Result<&'static str> {
		clear_screen()?;

		let mut show_deprecated = false;
		if self.features.is_some() && self.features.clone().unwrap().contains("e2e-tests") {
			show_deprecated = true;
			self.e2e = true;
		}

		if self.e2e {
			Cli.intro("Starting end-to-end tests")?;

			if show_deprecated {
				warning("NOTE: --features e2e-tests is deprecated. Use --e2e instead.")?;
				#[cfg(not(test))]
				sleep(Duration::from_secs(3)).await;
			}

			self.node = match check_contracts_node_and_prompt(self.skip_confirm).await {
				Ok(binary_path) => Some(binary_path),
				Err(_) => {
					warning("🚫 substrate-contracts-node is necessary to run e2e tests. Will try to run tests anyway...")?;
					Some(PathBuf::new())
				},
			};

			test_e2e_smart_contract(self.path.as_deref(), self.node.as_deref())?;
			outro("End-to-end testing complete")?;
			Ok("e2e")
		} else {
			Cli.intro("Starting unit tests")?;
			test_smart_contract(self.path.as_deref())?;
			outro("Unit testing complete")?;
			Ok("unit")
		}
	}
}

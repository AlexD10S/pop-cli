// SPDX-License-Identifier: GPL-3.0

use anyhow::Result;
use assert_cmd::{cargo::cargo_bin, Command};
use pop_common::templates::Template;
use pop_parachains::ParachainTemplate;
use std::{fs, path::Path, process::Command as Cmd};
use strum::VariantArray;
use tokio::time::{sleep, Duration};

/// Test the parachain lifecycle: new, build, up
#[tokio::test]
async fn parachain_lifecycle() -> Result<()> {
	let temp = tempfile::tempdir().unwrap();
	let temp_dir = temp.path();
	// let temp_dir = Path::new("./"); //For testing locally
	// Test that all templates are generated correctly
	generate_all_the_templates(&temp_dir)?;
	// pop new parachain test_parachain (default)
	Command::cargo_bin("pop")
		.unwrap()
		.current_dir(&temp_dir)
		.args(&[
			"new",
			"parachain",
			"test_parachain",
			"--symbol",
			"POP",
			"--decimals",
			"6",
			"--endowment",
			"1u64 << 60",
		])
		.assert()
		.success();
	assert!(temp_dir.join("test_parachain").exists());

	// pop build contract -p "./test_parachain"
	Command::cargo_bin("pop")
		.unwrap()
		.current_dir(&temp_dir)
		.args(&["build", "parachain", "-p", "./test_parachain", "--para_id", "2000"])
		.assert()
		.success();

	assert!(temp_dir.join("test_parachain/target").exists());
	// Assert build files has been generated
	assert!(temp_dir.join("test_parachain/raw-parachain-chainspec.json").exists());
	assert!(temp_dir.join("test_parachain/para-2000-wasm").exists());
	assert!(temp_dir.join("test_parachain/para-2000-genesis-state").exists());

	let content = fs::read_to_string(temp_dir.join("test_parachain/raw-parachain-chainspec.json"))
		.expect("Could not read file");
	// Assert the custom values has been set propertly
	assert!(content.contains("\"para_id\": 2000"));
	assert!(content.contains("\"tokenDecimals\": 6"));
	assert!(content.contains("\"tokenSymbol\": \"POP\""));

	// pop up contract -p "./test_parachain"
	let mut cmd = Cmd::new(cargo_bin("pop"))
		.current_dir(&temp_dir.join("test_parachain"))
		.args(&["up", "parachain", "-f", "./network.toml", "--skip-confirm", "--verbose"])
		.spawn()
		.unwrap();
	// If after 20 secs is still running probably execution is ok, or waiting for user response
	sleep(Duration::from_secs(20)).await;

	//assert!(cmd.try_wait().unwrap().is_none(), "the process should still be running");
	// Stop the process
	Cmd::new("kill").args(["-s", "TERM", &cmd.id().to_string()]).spawn()?;

	Ok(())
}

fn generate_all_the_templates(temp_dir: &Path) -> Result<()> {
	for template in ParachainTemplate::VARIANTS {
		let parachain_name = format!("test_parachain_{}", template);
		let provider = template.template_type()?.to_lowercase();
		// pop new parachain test_parachain
		Command::cargo_bin("pop")
			.unwrap()
			.current_dir(&temp_dir)
			.args(&[
				"new",
				"parachain",
				&parachain_name,
				&provider,
				"--template",
				&template.to_string(),
			])
			.assert()
			.success();
		assert!(temp_dir.join(parachain_name).exists());
	}
	Ok(())
}

// SPDX-License-Identifier: GPL-3.0

use anyhow::Result;
use assert_cmd::Command;
use pop_common::templates::Template;
use pop_contracts::{
	dry_run_gas_estimate_instantiate, instantiate_smart_contract, run_contracts_node,
	set_up_deployment, ContractTemplate, UpOpts,
};
use std::path::Path;
use strum::VariantArray;
use url::Url;

/// Test the contract lifecycle: new, build, test, up, call
#[tokio::test]
async fn contract_lifecycle() -> Result<()> {
	let temp = tempfile::tempdir().unwrap();
	let temp_dir = temp.path();
	//let temp_dir = Path::new("./"); //For testing locally
	// Test that all templates are generated correctly
	generate_all_the_templates(&temp_dir)?;
	// pop new contract test_contract (default)
	Command::cargo_bin("pop")
		.unwrap()
		.current_dir(&temp_dir)
		.args(&["new", "contract", "test_contract"])
		.assert()
		.success();
	assert!(temp_dir.join("test_contract").exists());

	// pop build contract
	Command::cargo_bin("pop")
		.unwrap()
		.current_dir(&temp_dir.join("test_contract"))
		.args(&["build", "contract", "--release"])
		.assert()
		.success();
	// Verify that the folder target has been created
	assert!(temp_dir.join("test_contract/target").exists());
	// Verify that all the artifacts has been generated
	assert!(temp_dir.join("test_contract/target/ink/test_contract.contract").exists());
	assert!(temp_dir.join("test_contract/target/ink/test_contract.wasm").exists());
	assert!(temp_dir.join("test_contract/target/ink/test_contract.json").exists());

	// pop test contract
	Command::cargo_bin("pop")
		.unwrap()
		.current_dir(&temp_dir.join("test_contract"))
		.args(&["test", "contract"])
		.assert()
		.success();
	// Run the contracts node
	let cache = temp_dir.join("cache");
	let mut process = run_contracts_node(cache).await?;
	// Only upload the contract
	// pop up contract --upload-only
	Command::cargo_bin("pop")
		.unwrap()
		.current_dir(&temp_dir.join("test_contract"))
		.args(&["up", "contract", "--upload-only", "-y"])
		.assert()
		.success();
	// Instantiate contract, only dry-run
	Command::cargo_bin("pop")
		.unwrap()
		.current_dir(&temp_dir.join("test_contract"))
		.args(&[
			"up",
			"contract",
			"--constructor",
			"new",
			"--args",
			"false",
			"--suri",
			"//Alice",
			"--dry-run",
			"-y",
		])
		.assert()
		.success();
	// Using methods from the pop_contracts crate to instantiate it to get the Contract Address for the call
	let instantiate_exec = set_up_deployment(UpOpts {
		path: Some(temp_dir.join("test_contract")),
		constructor: "new".to_string(),
		args: ["false".to_string()].to_vec(),
		value: "0".to_string(),
		gas_limit: None,
		proof_size: None,
		salt: None,
		url: Url::parse("ws://127.0.0.1:9944")?,
		suri: "//Alice".to_string(),
	})
	.await?;
	let weight_limit = dry_run_gas_estimate_instantiate(&instantiate_exec).await?;
	let contract_address = instantiate_smart_contract(instantiate_exec, weight_limit).await?;
	// Call contract (only query)
	// pop call contract --contract $INSTANTIATED_CONTRACT_ADDRESS --message get --suri //Alice
	Command::cargo_bin("pop")
		.unwrap()
		.current_dir(&temp_dir.join("test_contract"))
		.args(&[
			"call",
			"contract",
			"--contract",
			&contract_address,
			"--message",
			"get",
			"--suri",
			"//Alice",
		])
		.assert()
		.success();

	// Call contract (execute extrinsic)
	// pop call contract --contract $INSTANTIATED_CONTRACT_ADDRESS --message flip --suri //Alice -x
	Command::cargo_bin("pop")
		.unwrap()
		.current_dir(&temp_dir.join("test_contract"))
		.args(&[
			"call",
			"contract",
			"--contract",
			&contract_address,
			"--message",
			"flip",
			"--suri",
			"//Alice",
			"-x",
		])
		.assert()
		.success();

	// Kill the contracts node
	process.kill()?;

	Ok(())
}

fn generate_all_the_templates(temp_dir: &Path) -> Result<()> {
	for template in ContractTemplate::VARIANTS {
		let contract_name = format!("test_contract_{}", template);
		let contract_type = template.template_type()?.to_lowercase();
		// pop new parachain test_parachain
		Command::cargo_bin("pop")
			.unwrap()
			.current_dir(&temp_dir)
			.args(&[
				"new",
				"contract",
				&contract_name,
				"--contract-type",
				&contract_type,
				"--template",
				&template.to_string(),
			])
			.assert()
			.success();
		assert!(temp_dir.join(contract_name).exists());
	}
	Ok(())
}

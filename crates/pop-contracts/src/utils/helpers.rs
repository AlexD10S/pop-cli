// SPDX-License-Identifier: GPL-3.0

use crate::errors::Error;
use contract_build::ManifestPath;
use contract_extrinsics::BalanceVariant;
use ink_env::{DefaultEnvironment, Environment};
use std::{
	path::{Path, PathBuf},
	str::FromStr,
};
use subxt::{Config, PolkadotConfig as DefaultConfig};

pub fn get_manifest_path(path: Option<&Path>) -> Result<ManifestPath, Error> {
	if let Some(path) = path {
		let full_path = PathBuf::from(path.to_string_lossy().to_string() + "/Cargo.toml");
		return ManifestPath::try_from(Some(full_path))
			.map_err(|e| Error::ManifestPath(format!("Failed to get manifest path: {}", e)));
	} else {
		return ManifestPath::try_from(path.as_ref())
			.map_err(|e| Error::ManifestPath(format!("Failed to get manifest path: {}", e)));
	}
}

pub fn parse_balance(
	balance: &str,
) -> Result<BalanceVariant<<DefaultEnvironment as Environment>::Balance>, Error> {
	BalanceVariant::from_str(balance).map_err(|e| Error::BalanceParsing(format!("{}", e)))
}

pub fn parse_account(account: &str) -> Result<<DefaultConfig as Config>::AccountId, Error> {
	<DefaultConfig as Config>::AccountId::from_str(account)
		.map_err(|e| Error::AccountAddressParsing(format!("{}", e)))
}

/// Canonicalizes the given path to ensure consistency and resolve any symbolic links.
///
/// # Arguments
///
/// * `target` - A reference to the `Path` to be canonicalized.
///
pub fn canonicalized_path(target: &Path) -> Result<PathBuf, Error> {
	// Canonicalize the target path to ensure consistency and resolve any symbolic links.
	target
		.canonicalize()
		// If an I/O error occurs during canonicalization, convert it into an Error enum variant.
		.map_err(|e| Error::IO(e))
}

#[cfg(test)]
mod tests {
	use super::*;
	use anyhow::{Error, Result};
	use std::fs;

	async fn setup_test_environment() -> Result<tempfile::TempDir, Error> {
		let temp_dir = tempfile::tempdir().expect("Could not create temp dir");
		let temp_contract_dir = temp_dir.path().join("test_contract");
		fs::create_dir(&temp_contract_dir)?;
		crate::create_smart_contract(
			"test_contract",
			temp_contract_dir.as_path(),
			&crate::Contract::Standard,
		)
		.await?;
		Ok(temp_dir)
	}

	#[test]
	fn test_get_manifest_path() -> Result<(), Error> {
		let temp_dir = setup_test_environment()?;
		get_manifest_path(Some(&PathBuf::from(temp_dir.path().join("test_contract"))))?;
		Ok(())
	}

	#[test]
	fn test_canonicalized_path() -> Result<(), Error> {
		let temp_dir = tempfile::tempdir()?;
		// Error case
		let error_folder = canonicalized_path(&temp_dir.path().join("my_folder"));
		assert!(error_folder.is_err());
		// Success case
		canonicalized_path(temp_dir.path())?;
		Ok(())
	}
}

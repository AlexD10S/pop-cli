// SPDX-License-Identifier: GPL-3.0

use thiserror::Error;
use zombienet_sdk::OrchestratorError;

/// Represents the various errors that can occur in the crate.
#[derive(Error, Debug)]
pub enum Error {
	#[error("User aborted due to existing target directory.")]
	Aborted,
	#[error("Anyhow error: {0}")]
	AnyhowError(#[from] anyhow::Error),
	/// An error occurred while generating the chain specification.
	#[error("Failed to build the chain spec. {0}")]
	BuildSpecError(String),
	/// An error occurred while running benchmarking.
	#[error("Failed to run benchmarking: {0}")]
	BenchmarkingError(String),
	/// An error occurred while decoding the call data.
	#[error("Failed to decode call data. {0}")]
	CallDataDecodingError(String),
	/// An error occurred while encoding the call data.
	#[error("Failed to encode call data. {0}")]
	CallDataEncodingError(String),
	#[error("{0}")]
	CommonError(#[from] pop_common::Error),
	/// An error occurred while attempting to establish a connection to the endpoint.
	#[error("Failed to establish a connection to: {0}")]
	ConnectionFailure(String),
	#[error("Configuration error: {0}")]
	Config(String),
	#[error("Failed to access the current directory")]
	CurrentDirAccess,
	#[error("Failed to parse the endowment value")]
	EndowmentError,
	/// The specified event was not found.
	#[error("Event {0} not found.")]
	EventNotFound(String),
	/// An error occurred during the submission of an extrinsic.
	#[error("Extrinsic submission error: {0}")]
	ExtrinsicSubmissionError(String),
	/// The dispatchable function is not supported.
	#[error("The dispatchable function is not supported")]
	FunctionNotSupported,
	/// An error occurred while working with the genesis builder.
	#[error("Genesis builder error: {0}")]
	GenesisBuilderError(String),
	/// Failed to retrieve the image tag.
	#[error("Failed to retrieve image tag.")]
	ImageTagRetrievalFailed,
	#[error("IO error: {0}")]
	IO(#[from] std::io::Error),
	#[error("JSON error: {0}")]
	JsonError(#[from] serde_json::Error),
	/// An error occurred while parsing metadata of a parameter.
	#[error("Error parsing metadata for parameter {0}")]
	MetadataParsingError(String),
	#[error("Missing binary: {0}")]
	MissingBinary(String),
	#[error("Missing chain spec file at: {0}")]
	MissingChainSpec(String),
	#[error("Command {command} doesn't exist in binary {binary}")]
	MissingCommand { command: String, binary: String },
	#[error("Orchestrator error: {0}")]
	OrchestratorError(#[from] OrchestratorError),
	#[error("Failed to create pallet directory")]
	PalletDirCreation,
	/// The specified pallet could not be found.
	#[error("Failed to find the pallet {0}")]
	PalletNotFound(String),
	/// An error occurred while processing the arguments provided by the user.
	#[error("Failed to process the arguments provided by the user.")]
	ParamProcessingError,
	/// An error occurred while parsing the arguments provided by the user.
	#[error("Failed to parse the arguments provided by the user: {0}")]
	ParamParsingError(String),
	#[error("Invalid path")]
	PathError,
	#[error("Failed to execute rustfmt")]
	RustfmtError(std::io::Error),
	/// The specified runtime could not be found.
	#[error("Failed to find the runtime {0}")]
	RuntimeNotFound(String),
	#[error("Template error: {0}")]
	SourcingError(#[from] pop_common::sourcing::Error),
	/// An error occurred whilst interacting with a chain using `subxt`.
	#[error("Subxt error: {0}")]
	SubXtError(#[from] subxt::Error),
	#[error("Toml error: {0}")]
	TomlError(#[from] toml_edit::de::Error),
	#[error("Unsupported command: {0}")]
	UnsupportedCommand(String),
	#[error("Failed to locate the workspace")]
	WorkspaceLocate,
}

// Handles command execution errors by extracting and returning the stderr message using the
// provided error constructor.
pub(crate) fn handle_command_error<F>(
	output: &std::process::Output,
	custom_error: F,
) -> Result<(), Error>
where
	F: FnOnce(String) -> Error,
{
	if !output.status.success() {
		let stderr_msg = String::from_utf8_lossy(&output.stderr);
		return Err(custom_error(stderr_msg.to_string()));
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use anyhow::Result;
	use std::{
		os::unix::process::ExitStatusExt,
		process::{ExitStatus, Output},
	};

	#[test]
	fn handle_command_error_failure() -> Result<()> {
		let output = Output {
			status: ExitStatus::from_raw(1),
			stdout: Vec::new(),
			stderr: Vec::from("Error message".as_bytes()),
		};
		assert!(matches!(
			handle_command_error(&output, Error::BuildSpecError),
			Err(Error::BuildSpecError(message))
			if message == "Error message"
		));
		Ok(())
	}
}

# Pop CLI

<img src="/.icons/logo.jpeg"></img>

An all-in-one tool for Polkadot development.

## Install

You can install Pop CLI as follows:

```shell
cargo install --locked --git https://github.com/r0gue-io/pop-cli
```

> :information_source: For detailed instructions on how to install Pop CLI, please refer to our
> documentation: https://learn.onpop.io/v/contracts/welcome/installing-pop-cli
>
> A [crates.io](https://crates.io/crates/pop-cli) version will be available soon!

### Telemetry

Pop CLI collects anonymous usage metrics to help us understand how the tool is being used and how we can improve it.
We do not collect any personal information. If you wish to disable telemetry
or read more about our telemetry practices please see
our [telemetry](crates/pop-telemetry/README.md) documentation.

## Getting Started

### Parachains

Use `pop` to create a new Parachain project.
To be guided through the entire parachain creation process, simply execute

```sh
pop new parachain
```

If no guidance is needed, proceed with:

```sh
# Create a minimal parachain
pop new parachain my-app
```

`pop-cli` supports diverse project templates, to use a specific one use the flag `--template`:

```sh
# Create an assets parachain
pop new parachain my-app pop -t assets
# Create a contracts parachain
pop new parachain my-app pop -t contracts
# Create a evm parachain
pop new parachain my-app pop -t evm
```

We also integrate other provider templates in the tool, check them running:

```sh
pop new parachain --help
```

Some examples are:

```sh
# Get Parity's pallet-contracts enabled parachain template
pop new parachain my-app parity -t cpt
# Get Parity's evm compatible parachain template
pop new parachain my-app parity -t fpt
```

For Pop templates you can also customize your parachain by providing config options for token symbol (as it appears in
chain metadata), token decimals, and the initial endowment for developer accounts. Here's how:

```sh
# Create a minimal parachain with "DOT" as token symbol, 6 token decimals and 1 billion tokens per dev account
pop new parachain my-app --symbol DOT --decimals 6 --endowment 1_000_000_000
```

There's also the shorter version:

```sh
pop new parachain my-app -s DOT -d 6 -i 1_000_000_000
```

Use `pop` to build your Parachain:

```sh
# Build your parachain
pop build parachain -p ./my-app
```

or

```sh
cd my-app
pop build parachain
```

Generate the chain specification file and export the WASM and genesis state files when building your parachain with the `--para_id` flag:
```sh
pop build parachain -p ./my-app --para_id 2000
```

## Spawn Network using Zombienet

You can spawn a local network using [zombienet](https://github.com/paritytech/zombienet-sdk) as follows:

```shell
pop up parachain -f ./tests/networks/pop.toml -p https://github.com/r0gue-io/pop-node
```

> :information_source: Pop CLI will automatically source the necessary `polkadot` binaries.

Various examples of network configuration files are available [here](./tests/networks).

### Run a command after the network has been spun up

The following will spin up the network locally according the the zombienet file and once the network is up, it will run
the command specified in `--cmd`:

```shell
pop up parachain -f ./tests/networks/pop.toml --cmd ./path/to/my/script
```

### Contracts

Use `pop` to create a new Smart Contract project.
To be guided through the entire contract creation process, simply execute

```sh
pop new contract
```

If no guidance is needed, proceed with:

```sh
# Create a minimal Smart Contract
pop new contract my_contract
```

`pop-cli` supports different contract templates and can be used with`--template`:

```sh
# Create an ERC-20 standard in ink!
pop new contract my_contract -t erc20
# Create an ERC-721 standard in ink!
pop new contract my_contract -t erc721
# Create an ERC-1155 standard in ink!
pop new contract my_contract -t erc1155
```

Test the Smart Contract:

```sh
# Test an existing Smart Contract
pop test contract -p ./my_contract
```

Build the Smart Contract:

```sh
# Build an existing Smart Contract
pop build contract -p ./my_contract
```

By default the contract is compiled with `debug` functionality included.

This enables the contract to output debug messages, but increases the contract size and the amount of gas used.

For production builds, use the --release flag: `--release`:

```sh
pop build contract -p ./my_contract --release
```

Deploy and instantiate the Smart Contract:

```sh
pop up contract -p ./my_contract --constructor new --args "false" --suri //Alice
```

> :information_source: If you don't specify a live chain, `pop` will automatically spawn a local node for testing
> purposes.

Some of the options available are:

- Specify the contract `constructor `to use, which in this example is `new()`.
- Specify the argument (`args`) to the constructor, which in this example is `false`.
- Specify the account uploading and instantiating the contract with `--suri`, which in this example is the default
  development account of `//Alice`.
  For other accounts, the actual secret key must be provided e.g. an 0x prefixed 64 bit hex string, or the seed phrase.

> :warning: **Use only for development**: Use a safer method of signing here before using this feature with production
> projects. We will be looking to provide alternative solutions in the future!

- You also can specify the url of your node with `--url ws://your-endpoint`, by default it is
  using `ws://localhost:9944`.
- To perform a dry-run via RPC to estimate the gas usage without submitting a transaction use the `--dry-run` flag.

For more information about the options,
check [cargo-contract documentation](https://github.com/paritytech/cargo-contract/blob/master/crates/extrinsics/README.md#instantiate)

Interacting with the Smart Contract:

1. Read-only Operations: For operations that only require reading from the blockchain state. This approach does not
   require to submit an extrinsic.
   Example using the get() message:

```sh
pop call contract -p ./my_contract --contract $INSTANTIATED_CONTRACT_ADDRESS --message get --suri //Alice
```

2. State-modifying Operations: For operations that change a storage value, thus altering the blockchain state. Include
   the `x / --execute`  flag to submit an extrinsic on-chain.

Example executing the `flip()` message:

```sh
pop call contract -p ./my_contract --contract $INSTANTIATED_CONTRACT_ADDRESS --message flip --suri //Alice -x
```

## E2E testing

For end-to-end testing you will need to have a Substrate node with `pallet contracts`.
You do not need to run it in the background since the node is started for each test independently.
To install the latest version:
```
cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git
```

Run e2e testing on the Smart Contract:

```sh
# Run e2e tests for an existing smart contract
 pop test contract  -p ./my_contract --features e2e-tests
```

If you want to run a different node with `pallet-contracts` you need to change `CONTRACTS_NODE` environment variable using the  `--node` flag:
```sh
# Run e2e tests for an existing smart contract
 pop test contract  -p ./my_contract --features e2e-tests --node YOUR_CONTRACTS_NODE_PATH
```

### Pallets

To create a new Pallet, simply run `pop new pallet`. You will have a new pallet ready for hacking.
To customize the new Pallet you can follow these options:

```sh
# create a pallet with name `pallet-awesome` in the current working directory
pop new pallet pallet-awesome
# or with options
pop new pallet pallet-awesome --authors Me --description "This pallet oozes awesomeness" --path my_app/pallets
```

## Building Pop CLI locally

Build the tool locally with all the features:

```sh
cargo build --all-features
```

Build the tool only for Parachain functionality:

```sh
cargo build --features parachain
```

Build the tool only for Smart Contracts functionality:

```sh
cargo build --features contract
```

## Testing Pop CLI

To test the tool locally. Due to the time it can take to build a Parachain or a Smart Contract, some tests have been
separated from the normal testing flow into integration tests.

Run the unit tests only:

```sh
cargo test --lib
```

To run the integration tests relating to Smart Contracts:

```sh
cargo test --test contract
```

To run the integration tests relating to Parachains:

```sh
cargo test --test parachain
```

Run all tests (unit + integration):

```sh
cargo test
```

## Acknowledgements

Pop CLI would not be possible without these awesome crates!

- Local network deployment powered by [zombienet-sdk](https://github.com/paritytech/zombienet-sdk)
- [cargo contract](https://github.com/use-ink/cargo-contract) a setup and deployment tool for developing Wasm based
  Smart Contracts via ink!

## License

The entire code within this repository is licensed under the [GPLv3](LICENSE).

Please [contact us](https://r0gue.io/contact) if you have questions about the licensing of our products.

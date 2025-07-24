# Cli

## Installation

Install with cargo:

```bash
cargo install --git https://github.com/wakeuplabs-io/noir-stylus-verifier --bin nsv --branch develop
```

You can also download already compiled binaries from the github releases at [noir-stylus-verifier](https://github.com/wakeuplabs-io/noir-stylus-verifier)

## Commands

```
nsv --help

Generate and deploy verifiers in stylus from noir circuits.

Usage: nsv [OPTIONS] <COMMAND>

Commands:
  new       Create a new project
  generate  Generate a verifier contract from a noir circuit
  check     Check if the generated contract is compatible with Stylus, and how much it costs to deploy
  deploy    Deploy the generated contract to the blockchain
  prove     Generate a proof for a circuit. Useful for testing
  verify    Verify a proof for a circuit. Useful for testing
  help      Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose  Suppress logging output
  -h, --help     Print help
  -V, --version  Print version
```

### `new`

```
> nsv new --help

Create a new project

Usage: nsv new <TARGET>

Arguments:
  <TARGET>  Name of the project. This will also be the directory and package name.

Options:
  -h, --help  Print help
```

### `generate`

```
> nsv generate --help

Generate a verifier contract from a noir circuit

Usage: nsv generate [OPTIONS]

Options:
  -p, --package <PACKAGE>              Package name containing the circuit
      --bytecode-path <BYTECODE_PATH>  Path to the bytecode to use for the proof generation
      --vk-path <VK_PATH>              Path to the verification key to use for the proof generation
  -h, --help 
```

### `check`

```
> nsv check --help

Check if the generated contract is compatible with Stylus, and how much it costs to deploy

Usage: nsv check [OPTIONS]

Options:
  -p, --package <PACKAGE>  Package name containing the circuit
      --rpc-url <RPC_URL>  RPC URL to use for the check [default: https://sepolia-rollup.arbitrum.io/rpc]
  -h, --help               Print help
```

### `deploy`

```
> nsv deploy --help

Deploy the generated contract to the blockchain

Usage: nsv deploy [OPTIONS] --rpc-url <RPC_URL> --private-key <PRIVATE_KEY>

Options:
  -p, --package <PACKAGE>
          Package name containing the circuit
      --rpc-url <RPC_URL>
          RPC URL to use for deployment
      --private-key <PRIVATE_KEY>
          Private key to sign the deployment transaction
      --verifier-address <VERIFIER_ADDRESS>
          Address of the global verifier contract. Optional if using defaults (see `docs/deployments.md`).
      --zk
          Enable zk-flavored verifier
  -h, --help
          Print help
```

### `prove`

```
> nsv prove --help

Generate a proof for a circuit. Useful for testing

Usage: nsv prove [OPTIONS]

Options:
  -p, --package <PACKAGE>              Package name containing the circuit
      --prover-name <PROVER_NAME>      Name of the prover to use for the proof generation [default: Prover.toml]
      --output-path <OUTPUT_PATH>      Path to the proof to generate [default: target/proof]
      --witness-path <WITNESS_PATH>    Path to the witness to use for the proof generation
      --bytecode-path <BYTECODE_PATH>  Path to the bytecode to use for the proof generation
      --zk                             Enable zk-flavored proof
  -h, --help                           Print help
```

### `verify`

```
> nsv verify --help

Verify a proof for a circuit. Useful for testing

Usage: nsv verify [OPTIONS]

Options:
      --proof <PROOF>
          Path to the proof to verify [default: target/proof]
      --public-input <PUBLIC_INPUT>
          Path to the public input to verify [default: target/public_inputs]
      --vk <VK>
          Path to the verification key [default: contracts/assets/vk]
      --verifier-address <VERIFIER_ADDRESS>
          Address of the deployed verifier contract (defaults to local verifier if omitted)
      --rpc-url <RPC_URL>
          RPC URL to use for verification
      --zk
          Set if using a zk-flavored verifier and proof
  -h, --help
          Print help
```

## Development

To build the cli binary for distribution first add the necessary rust targets:

```bash
brew install mingw-w64
cargo install --locked cargo-zigbuild

rustup target add x86_64-apple-darwin
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-unknown-linux-musl
```

Available just recipes:

```bash
just --summary

# Available recipes:
#     build-cli-linux                         # Build and package the cli binary for linux
#     build-cli-macos                         # Build and package the cli binary for macos
#     build-cli-windows                       # Build and package the cli binary for windows
#     clean-cli-linux                         # Clean the cli binary for linux
#     clean-cli-macos                         # Clean the cli binary for macos
#     clean-cli-windows                       # Clean the cli binary for windows
#     test-cli                                # Test the cli unit tests
#     test-cli-integration                    # Run the cli integration tests.
```
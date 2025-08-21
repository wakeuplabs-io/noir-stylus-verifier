
## Contribute

### Cloning the repo

```bash
git clone --recurse-submodules https://github.com/wakeuplabs-io/noir-stylus-verifier.git
```

### Folder Structure

- **`cli/`**
  Command line interface for generating and deploying verifier contracts from Noir circuits.
  See `docs/cli.md` for detailed usage instructions.
  Key commands:

  - `nsv new` - Create new project
  - `nsv generate` - Generate verifier contract
  - `nsv check` - Check contract compatibility
  - `nsv deploy` - Deploy to chain
  - `nsv prove` - Generate proof
  - `nsv verify` - Verify proof

- **`contracts/`**
  Contains Stylus verifier contracts.

- **`docs/`**
  All documentation lives here.
  → To propose changes, open a PR or issue as outlined in `docs/contribute.md`.

- **`examples/`**
  End-to-end examples demonstrating usage of the Stylus verifier.

- **`integration/`**
  CLI package to run integration tests, compile/deploy contracts, and send test transactions.
  Supports local node, sepolia, or mainnet by passing appropriate parameters.

- **`packages/ultrahonk/`**
  Core Ultrahonk verifier logic, based on the [TACEO implementation](https://github.com/TaceoLabs/co-snarks) which closely follows original [BB implementation](https://github.com/AztecProtocol/aztec-packages/tree/master/barretenberg/cpp/src/barretenberg/ultra_honk)

- **`scripts/`**
  Assorted bash scripts for setup, builds, and automation.

- **`test_vectors/`**
  Noir circuits and associated artifacts (`vk`, `proof`, `public_inputs`) used for testing compatibility with Barretenberg.
  Generated via:

  ```bash
  ./scripts/compile-test-vectors.sh
  ```

  If test vectors change, update:

  - `./integration/src/tests/verifier.rs`
  - `./packages/ultrahonk/tests/verifier.rs`

### Contracts

Due to Stylus size limits on Arbitrum, the Ultrahonk verifier is split into **three contracts**:

1. **Verifier**
   Deserializes inputs, builds the transcript and verifier memory, then coordinates calls to SumcheckVerifier and ShpleminiVerifier. In ZK mode, it also checks evaluation consistency.

2. **Sumcheck Verifier**
   Implements the Ultrahonk sumcheck protocol.

3. **Shplonk (Shplemini) Verifier**
   Implements Shplemini logic. Requires prior execution of sumcheck to initialize memory/transcript state.

Same applies for ZK flavoured options

#### Verification Options

Verification calls are **readonly** (no gas cost), and you have three main integration paths:

1. **Global Verifier (Recommended for quick testing)**

   - See: `examples/hello_world/scripts/verify-global.js`
   - Requires passing the verification key and public inputs each time.
   - Public inputs must be serialized manually on the frontend.

2. **Circuit-Specific Wrapper Contracts**

   - See: `examples/hello_world/contracts`
   - These hardcode the verification key and simplify the call interface.

3. **Directly Integrate `ultrahonk` Package**

   - Embed the logic in your own contracts.
   - May hit size limits, but gives full control.

#### Flavors

We currently support two Ultrahonk configurations, controlled via Cargo feature branches:

- **UltraKeccakFlavor**:

  - `verifier`, `sumcheck-verifier`, `shplemini-verifier`

- **UltraKeccakZKFlavor**:

  - `zk-verifier`, `zk-sumcheck-verifier`, `zk-shplemini-verifier`

  
### Requirements

#### Rust (nightly-2025-06-12)

Install the required nightly toolchain and WASM target:

```bash
rustup toolchain install nightly-2025-06-12 --component rust-src
rustup target add wasm32-unknown-unknown --toolchain nightly-2025-06-12
```

#### Just (1.38.0)

Used to run project commands:

```bash
cargo install just@1.38.0
```

#### Cargo Stylus (0.6.0)

Build and deploy Stylus contracts:

```bash
cargo install cargo-stylus@0.6.3
```

#### Noir (1.0.0-beta.6)

Install Noir for ZK circuit work:

```bash
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup -v 1.0.0-beta.6
```

#### Barretenberg (0.86.0)

Required for proof generation and verification:

```bash
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/master/barretenberg/bbup/install | bash
bbup -v 0.86.0
```

#### Docker

Used primarily for running integration tests.

➡️ Install from: [https://docs.docker.com/engine/install/](https://docs.docker.com/engine/install/)

### Available scripts

There're a bunch of scripts for development available in the `justfile`

#### Global Configuration

You can override the following global variables:

* `rpc_url`: the RPC endpoint to interact with.
* `private_key`: the private key used for signing transactions.

By default, these point to a local testnode. To override them, use:

```bash
just --set rpc_url {your_rpc_url} --set private_key {your_private_key} {your_command}
```

Most commands will automatically pull these values from the global configuration. If not set, they default to the local testnode.

#### Contract Selection

Stylus allows only **one contract per WASM file**. To keep all contracts within the same `contracts` package, we've split them across different feature branches. You can specify any of the following as the `contract` variable:

* `verifier`
* `zk-verifier`
* `sumcheck-verifier`
* `zk-sumcheck-verifier`
* `shplemini-verifier`
* `zk-shplemini-verifier`

Use the appropriate one depending on your target logic.

####  Available recipes

```bash
just --summary

# Available recipes:
#     build-cli-linux                         # Build and package the cli binary for linux
#     build-cli-macos                         # Build and package the cli binary for macos
#     build-contract contract                 # Build the contracts in wasm for stylus. "Contract" is the feature flag within the contracts crate.
#     build-ultrahonk                         # Build the ultrahonk library for wasm target with only-arithmetic-backend feature.
#     check-contract contract                 # Check the contracts for deployment using stylus. "Contract" is the feature flag within the contracts crate.
#     check-pr                                # Check the pr.
#     clean-cli-linux                         # Clean the cli binary for linux
#     clean-cli-macos                         # Clean the cli binary for macos
#     deploy-contract contract constructor_signature="" *constructor_args="" # Deploy the contracts using stylus. "Contract" is the feature flag within the contracts crate. Pass constructor_signature and constructor_args to deploy with a constructor.
#     fmt                                     # Check formatting issues
#     fmt-fix                                 # Format the code and fix the errors.
#     get-verifier-addresses verifier_address # Get the verifier addresses for the global verifier.
#     lint                                    # Check linting issues
#     lint-fix                                # Fix linting issues
#     nitro-testnode-down                     # Quit the nitro testnode.
#     nitro-testnode-up                       # Spin up the nitro testnode.
#     profile-contract contract               # Profile the contracts using twiggy. "Contract" is the feature flag within the contracts crate.
#     test-cli                                # Test the cli unit tests
#     test-cli-integration                    # Run the cli integration tests.
#     test-contracts-integration              # Run the contracts integration tests.
#     test-integration                        # Run the integration tests. Spin up devnode and run. We'll exit with 0 if all tests pass, 1 otherwise, but still run them all.
#     test-ultrahonk-integration              # Run the ultrahonk integration tests.
#     verify-proof verifier_address test_vector_name zk="false" # Verify a proof from the test vectors with cast. Intended for global verifier only.
```

## How Can I Contribute?

If you find a security vulnerability, or are not sure whether it is a security vulnerability, _DO NOT OPEN A GITHUB ISSUE_. Read the section on how to handle [security vulnerabilities](#security-vulnerabilities).

### Reporting Bugs

If you find a bug, please [open an issue](https://github.com/wakeuplabs-io/noir-stylus-verifier/issues) and provide as much detail as possible. Make sure to include:

- A clear and descriptive title.
- A detailed description of the problem, including any error messages.
- Steps to reproduce the issue.
- The expected and actual behavior.
- Environment details (operating system, Rust version, etc.).
  
### Suggesting Enhancements

If you have an idea for a new feature or an improvement to an existing feature, we’d love to hear from you! Please [open an issue](https://github.com/wakeuplabs-io/noir-stylus-verifier/issues) and include:

- A clear and descriptive title.
- A detailed explanation of the proposed enhancement.
- Any relevant examples, code snippets, or use cases.

### Submitting Code Changes

Before you start working on a new feature or a bug fix, please check the open issues and confirm that the work is not already in progress. If it’s a significant change, it might be worth discussing your idea with the maintainers first.

#### Guidelines

Upon each PR we run github actions to ensure compliance, that includes linting, building and testing. All these use the same `justfile` available at the root. Before rising a PR make sure everything to run them all locally. 

```bash
just check-pr
```

Keep the following things in mind:

- **Follow Rust clippy**: We follow the guidelines from clippy.
- **Public API must be documented**: every exposed artifact must be documented.
- **Keep commits atomic**: Each commit should be a self-contained piece of work, with a clear commit message. The commit message should follow the guidelines for [conventional commit message](https://www.conventionalcommits.org/en/v1.0.0/).

### Writing Tests

Tests are essential for maintaining the reliability of the project. Please make sure that:

- All new features include unit/e2e tests.
- Bug fixes include regression tests to prevent future issues.
- The entire test suite passes before submitting your changes.

### Improving Documentation

Clear and comprehensive documentation helps others understand how to use and contribute to the project. You can contribute by:

- Fixing typos or improving explanations in existing documentation.
- Adding documentation for new features.
- Improving examples and tutorials.

## Security Vulnerabilities

If you find a security vulnerability, DO NOT open an issue on GitHub. Instead, please email the details to [WakeUp](mailto:contact@wakeuplabs.io).

We take security vulnerabilities seriously and will respond promptly to address the issue.

## Pull Request Process

1. Fork the repository and create a new branch for your feature or bug fix:

    ```bash
    git checkout -b your-feature-branch
    ```

2. Make your changes in the feature branch.
3. Run the linter and test suite with:

    ```bash
    just check-pr
    ```

4. Commit your changes with a [clear and descriptive message](https://www.conventionalcommits.org/en/v1.0.0/).
5. Push to your fork and open a pull request against the main branch.
6. Your pull request will be reviewed, and feedback may be provided. Once approved, it will be merged into the main branch.

## Attribution

This contribution guide was heavily inspired by TACEO's contribution guidelines.


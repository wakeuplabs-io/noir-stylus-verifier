
## Contribute

### Global Configuration

You can override the following global variables:

* `rpc_url`: the RPC endpoint to interact with.
* `private_key`: the private key used for signing transactions.

By default, these point to a local testnode started with:

```bash
just nitro-testnode-up
```

To override them, use:

```bash
just --set rpc_url {your_rpc_url} --set private_key {your_private_key} {your_command}
```

Most commands will automatically pull these values from the global configuration. If not set, they default to the local testnode.

### Contract Selection

Stylus allows only **one contract per WASM file**. To keep all contracts within the same `contracts` package, we've split them across different feature branches. You can specify any of the following as the `contract` variable:

* `verifier`
* `zk-verifier`
* `sumcheck-verifier`
* `zk-sumcheck-verifier`
* `shplemini-verifier`
* `zk-shplemini-verifier`

Use the appropriate one depending on your target logic.


### Build and Profile

Scripts to build and analyze the project.

* `just build-all`:
  Build all packages in the project.

* `just build-ultrahonk`:
  Build the `ultrahonk` package.

* `just build-contract {contract}`:
  Build the Stylus contract with the given `{contract}` feature enabled.
  The generated WASM file will be renamed accordingly.

* `just profile-contract {contract}`:
  Analyze the built WASM using `twiggy`.

  > ⚠️ Make sure to:
  >
  > * Run the build step first (`just build-contract {contract}`).
  > * Set `strip = "none"` in your `Cargo.toml` `[profile.release]` section to preserve symbol information.
  >   The output will be saved to `profile/{contract}...`.


### Tests

Scripts for testing the project and verifying proofs.

* `just test-integration`:
  Runs integration tests.

  > Requires either:
  >
  > * A local Nitro testnode (`just nitro-testnode-up`), or
  > * Specifying `rpc_url` and `private_key` for an Arbitrum testnet/mainnet.

* `just test-ultrahonk`:
  Runs tests for the `ultrahonk` package.

* `just verify-proof {verifier-address} {test-vector}`:
  Calls the on-chain verifier at `{verifier-address}` using the `vk`, `proof`, and `public_inputs` from:

  ```bash
  test_vectors/{test-vector}/kat/...
  ```

* `just get-verifier-addresses {verifier_address}`:
  Retrieves derived addresses for the `shplemini` and `sumcheck` verifiers based on `{verifier_address}`.


### Deployments

Scripts for checking and deploying contracts.

* `just check-contract {contract}`:
  Verifies that the contract is valid using `cargo stylus` and a Sepolia RPC endpoint.

* `just deploy-contract {contract} {constructor-signature} {constructor-args}`:
  Builds and deploys the specified Stylus contract with the given constructor.
  Example:

  ```bash
  just deploy-contract zk-verifier "constructor(address)" 0xabc123...
  ```

### Miscellaneous

Utility commands for development and local testing.

* `just nitro-testnode-up`:
  Spin up a local Nitro testnode for development.

* `just nitro-testnode-down`:
  Shut down the local Nitro testnode.

* `fmt`:
  Check code formatting (uses `cargo fmt` in check mode).

* `fmt-fix`:
  Automatically fix formatting issues.

* `lint`:
  Run `clippy` to check for linter warnings and best practices.

* `lint-fix`:
  Apply automatic `clippy` fixes where possible.

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


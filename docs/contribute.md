
## Contribute

## Development

Global variables you can override:
- `rpc_url`
- `private_key`

By default these point to a local testnode started with `just nitro-testnode-up`. To overwrite them use: `just --set rpc_url {...} --set private_key {...} {command as usual}`

Down below we'll specify all scripts available for development and management of the project. Remember these variables are global, most commands that need them will pull the global config that by default is local testnode. 

Also, stylus allows just one contract per wasm file, to keep them all together in the same contracts package we split them in different feature branches: `verifier`, `sumcheck-verifier` and `shplemini-verifier`. These are the available options for `contract` everywhere you see that as a variable.

### Build and profile

Some build scripts for the project.

- `just build-all` 
- `just build-ultrahonk`: Build ultrahonk package
- `just build-contract {contract}`: Build the contract package with `{contract}` as feature and renames the generated wasm accordingly.
- `just profile-contract {contract}`: Uses twiggy to further analyze the generated wasm. You need to set `strip = "none"` in `Cargo.toml` release target to get some readable output from this. Generated analyses will be in `profile/{contract}...`

### Tests 

- `just test-integration`: Requires having the nitro testnode running locally or specifying rpc_url and private key for some testnet/mainnet on arbitrum.
- `just test-ultrahonk`: Runs tests for ultrahonk package
- `just verify-proof {verifier-address} {test-vector}`: Calls the verifier with the `vk`, `proof` and `public_inputs` specified in `test_vectors/{test-vector}/kat/...`
- `just get-verifier-addresses {verifier_address}`: Retrieves addresses for shplemini and sumcheck verifiers.


### Deployments

- `just check-contract {contract}`: Uses sepolia rpc and cargo stylus cli to verify the validity of the contract
- `just deploy-contract {contract} {constructor-signature} {constructor-args}`: Builds and deploys the specified contract.


### Miscellaneous

- `just nitro-testnode-up`: Spin up testnet node
- `just nitro-testnode-down`: Shut down the nitro testnode
- `fmt`: Format all with cargo (write mode)
- `lint`: Check fmt issues and run clippy for other recommendations.

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


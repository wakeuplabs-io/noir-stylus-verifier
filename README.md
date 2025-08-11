# Noir Stylus Verifier

[![Build Status](https://github.com/wakeuplabs-io/noir-stylus-verifier/actions/workflows/rust-build.yml/badge.svg)](https://github.com/wakeuplabs-io/noir-stylus-verifier/actions/workflows/rust-build.yml)

> 🚧 Work in progress – things may break or change.

A Stylus-compatible UltraHonk verifier that bridges Noir's zero-knowledge capabilities with Arbitrum Stylus, enabling efficient verification of Barretenberg proofs in a WASM environment.

This project surges as a [proposal](https://github.com/orgs/noir-lang/discussions/8673) to Aztec [discussion](https://github.com/orgs/noir-lang/discussions/8345)

Find out everything about the project at the [docs folder](/docs/)

## Cloning the repo

```bash
git clone --recurse-submodules https://github.com/wakeuplabs-io/noir-stylus-verifier.git
```

# Specification

## Folder Structure

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

## Contracts

Due to Stylus size limits on Arbitrum, the Ultrahonk verifier is split into **three contracts**:

1. **Verifier**
   Deserializes inputs, builds the transcript and verifier memory, then coordinates calls to SumcheckVerifier and ShpleminiVerifier. In ZK mode, it also checks evaluation consistency.

2. **Sumcheck Verifier**
   Implements the Ultrahonk sumcheck protocol.

3. **Shplonk (Shplemini) Verifier**
   Implements Shplemini logic. Requires prior execution of sumcheck to initialize memory/transcript state.

Same applies for ZK flavoured options

### Verification Options

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

### Flavors

We currently support two Ultrahonk configurations, controlled via Cargo feature branches:

- **UltraKeccakFlavor**:

  - `verifier`, `sumcheck-verifier`, `shplemini-verifier`

- **UltraKeccakZKFlavor**:

  - `zk-verifier`, `zk-sumcheck-verifier`, `zk-shplemini-verifier`

  
## Requirements

### Rust (nightly-2025-06-12)

Install the required nightly toolchain and WASM target:

```bash
rustup toolchain install nightly-2025-06-12 --component rust-src
rustup target add wasm32-unknown-unknown --toolchain nightly-2025-06-12
```

### Just (1.38.0)

Used to run project commands:

```bash
cargo install just@1.38.0
```

### Cargo Stylus (0.6.0)

Build and deploy Stylus contracts:

```bash
cargo install cargo-stylus@0.6.0
```

### Noir (1.0.0-beta.6)

Install Noir for ZK circuit work:

```bash
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup -v 1.0.0-beta.6
```

### Barretenberg (0.86.0)

Required for proof generation and verification:

```bash
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/master/barretenberg/bbup/install | bash
bbup -v 0.86.0
```

### Docker

Used primarily for running integration tests.

➡️ Install from: [https://docs.docker.com/engine/install/](https://docs.docker.com/engine/install/)

## Licenses and Attribution

This project includes third-party code licensed under:

- MIT License — © 2024 TACEO
- Apache License 2.0 — See LICENSE file for full terms
- GNU GPL v3.0 — See LICENSE-GPL for full terms

All original licenses are preserved. See individual source files and LICENSE files for details.

Modifications made by © 2025 WakeUp

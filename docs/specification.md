# Specification

## Folder Structure

* **`cli/`**
  Command line interface for generating and deploying verifier contracts from Noir circuits.
  See `docs/cli.md` for detailed usage instructions.
  Key commands:
  - `nsv new` - Create new project
  - `nsv generate` - Generate verifier contract
  - `nsv check` - Check contract compatibility
  - `nsv deploy` - Deploy to chain
  - `nsv prove` - Generate proof
  - `nsv verify` - Verify proof

* **`contracts/`**
  Contains Stylus verifier contracts.

* **`docs/`**
  All documentation lives here.
  → To propose changes, open a PR or issue as outlined in `docs/contribute.md`.

* **`examples/`**
  End-to-end examples demonstrating usage of the Stylus verifier.

* **`integration/`**
  CLI package to run integration tests, compile/deploy contracts, and send test transactions.
  Supports local node, sepolia, or mainnet by passing appropriate parameters.

* **`packages/ultrahonk/`**
  Core Ultrahonk verifier logic, based on the [TACEO implementation](https://github.com/TaceoLabs/co-snarks) which closely follows original [BB implementation](https://github.com/AztecProtocol/aztec-packages/tree/master/barretenberg/cpp/src/barretenberg/ultra_honk)

* **`scripts/`**
  Assorted bash scripts for setup, builds, and automation.

* **`test_vectors/`**
  Noir circuits and associated artifacts (`vk`, `proof`, `public_inputs`) used for testing compatibility with Barretenberg.
  Generated via:

  ```bash
  ./scripts/compile-test-vectors.sh
  ```

  If test vectors change, update:

  * `./integration/src/tests/verifier.rs`
  * `./packages/ultrahonk/tests/verifier.rs`


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

   * See: `examples/hello_world/scripts/verify-global.js`
   * Requires passing the verification key and public inputs each time.
   * Public inputs must be serialized manually on the frontend.

2. **Circuit-Specific Wrapper Contracts**

   * See: `examples/hello_world/contracts`
   * These hardcode the verification key and simplify the call interface.

3. **Directly Integrate `ultrahonk` Package**

   * Embed the logic in your own contracts.
   * May hit size limits, but gives full control.

### Flavors

We currently support two Ultrahonk configurations, controlled via Cargo feature branches:

* **UltraKeccakFlavor**:

  * `verifier`, `sumcheck-verifier`, `shplemini-verifier`

* **UltraKeccakZKFlavor**:

  * `zk-verifier`, `zk-sumcheck-verifier`, `zk-shplemini-verifier`


## Packages

* **`packages/ultrahonk/`**
  Core logic for the Ultrahonk verifier.

* **`vendor/algebra/`**
  Forked from `arkworks/algebra`, modified to enable the `only-arithmetic-backend` feature.
  Ensures elliptic curve operations (`ec_add`, `ec_scalar_mul`, `ec_pairing_check`, `msm`) route through Stylus adapters instead of directly through Arkworks.

* **`integration/`**
  CLI interface for orchestrating integration tests and deployments.

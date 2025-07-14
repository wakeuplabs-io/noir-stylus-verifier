
# Specification

## Folder structure

- `contracts`: Here you'll find the verifier contract on itself, we deploy this as a global verifier.
- `docs`: You'll find all you need to know about the project in this folder. If there's something you'll like to see feel free to create a pr or raise and issue as stated in `docs/contribute.md`.
- `examples`: Find examples using the stylus verifier from end to end.
- `integration`: Cli package uses to run integration tests, compile and deploy contracts into local node (optionally sepolia or even mainnet by parameters), then execute the test transactions for precompiles module and verifier.
- `packages/ultrahonk`: Core ultrahonk verifier logic, based on TACEO implementation. 
- `scripts`: Some bash scripts for the project
- `test_vectors`: Bunch of noir circuits for which `vk`, `proof` and `public_inputs` are generated with the `bb` cli tool to use in tests proving compatibility. `vk`, `proof` and `public_inputs` are generated automatically with `./scripts/compile-test-vectors.sh`. If more cases are added or extracted you should update `./integration/src/tests/verifier.rs` as well as `./packages/ultrahonk/tests/verifier.rs`

## Contracts

Due to size limitations on arbitrum stylus we splitted the ultrahonk verifier in 3 contracts:
1. Verifier: Receives proof, builds initial transcript and memory and coordinates calls with shplemini and sumcheck verifier to return a true or false result.
2. Sumcheck verifier runs sumcheck verifications as per ultrahonk especification.
3. Shplemini verifier runs shplemini verifications as per ultrahonk especification, and needs sumcheck to be run before hand to receive an updated transcript and memory.

Verify calls are readonly therefore developers can choose between a few paths:
1. Use the Global verifier deployed as showned in `examples/hello_world/scripts/verify-global.js`. Virtually free as no extra deployment is required, but requires passing down the verification key every time and custom serializing the public inputs in the frontend.
2. Generate a verifier contract particular to the circuit that uses the global verifier behind the curtains as shown in `examples/hello_world/contracts`. This one already includes the verification key in the contract and allows for a more friendly contract call.
3. Pull in the `ultrahonk` package all toghether and use the verifier directly. Developers may struck size limitations as well deriving in multiple contracts, but it would all be under their domain. 

## Packages

- `packages/ultrahonk`: Contains the core ultrahonk verifier logic.
- `packages/ark-ec`: A copy from `arkworks/ec` that implements `only-arithmetic-backend` feature to verify `ultrahonk` package does `ec_add`, `ec_scalar_mul`, `ec_pairing_check` and `msm` though the adapater and not using the arkworks package.
- `integration`: Runs integration tests

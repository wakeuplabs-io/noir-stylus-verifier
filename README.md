# Noir Stylus Verifier

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/wakeuplabs-io/noir-stylus-verifier/actions/workflows/rust_build.yml/badge.svg)](https://github.com/wakeuplabs-io/noir-stylus-verifier/actions)

A Stylus-compatible UltraHonk verifier that bridges Noir's zero-knowledge capabilities with Arbitrum Stylus, enabling efficient verification of Barretenberg proofs in a WASM environment.

## Overview

This project provides a production-ready verifier for Noir proofs on Arbitrum Stylus, addressing the current gap in native verification support. By leveraging Stylus's WebAssembly environment and Ethereum precompiles, we enable efficient and secure verification of zero-knowledge proofs.

## Deployments

Deploy with:

```bash 
# deploy
just deploy-contract sumcheck-verifier
just deploy-contract shplemini-verifier
just deploy-contract verifier "constructor(address,address)" "{sumcheck-verifier}" "{shplemini-verifier}"

# And verify with
just get-verifier-addresses "{verifier}"
just verify-proof "{verifier}" "{test_vector_name}"
```

Sepolia

```bash
TODO:
```

## Usage

Create your noir circuit as usual and then do:

```bash
nargo execute
bb write_vk --oracle_hash keccak -o target -b target/{}.json
bb prove -b ./target/{}.json -w ./target/{}.gz -o ./target --scheme ultra_honk --oracle_hash keccak
```

## Project requirements

Noir:

```bash
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup -v 1.0.0-beta.6
```

Barretenberg 0.86.0:

```bash
curl -L curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/master/barretenberg/bbup/install | bash
bbup -v 0.86.0
```

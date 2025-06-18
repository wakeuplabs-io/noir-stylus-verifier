#!/bin/bash
set -e

# Navigate to project root
cd "$(dirname "$(realpath "$0")")/.."

export RPC_URL=http://localhost:8547
export DEPLOYER_ADDRESS=0x6ac4839Bfe169CadBBFbDE3f29bd8459037Bf64e

# test e2e backends

cargo build -p contracts --release --target wasm32-unknown-unknown  --features e2e-backends

# If any arguments are set, just pass them as-is to the cargo test command
if [[ $# -eq 0 ]]; then
    cargo test -p contracts --features e2e-backends --test "*"
else
    cargo test -p contracts --features e2e-backends "$@"
fi  

# test e2e verifier

cargo build -p contracts --release --target wasm32-unknown-unknown  --features verifier

# If any arguments are set, just pass them as-is to the cargo test command
if [[ $# -eq 0 ]]; then
    cargo test -p contracts --features verifier --test "*"
else
    cargo test -p contracts --features verifier "$@"
fi  
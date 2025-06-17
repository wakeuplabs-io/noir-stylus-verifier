#!/bin/bash
set -e

# Navigate to project root
cd "$(dirname "$(realpath "$0")")/.."

cargo build -p contracts --release --target wasm32-unknown-unknown  --features hash-precompile,e2e-precompile

export RPC_URL=http://localhost:8547
export DEPLOYER_ADDRESS=0x6ac4839Bfe169CadBBFbDE3f29bd8459037Bf64e

# If any arguments are set, just pass them as-is to the cargo test command
if [[ $# -eq 0 ]]; then
    cargo test -p contracts --features hash-precompile --test "*"
else
    cargo test -p contracts --features hash-precompile "$@"
fi  
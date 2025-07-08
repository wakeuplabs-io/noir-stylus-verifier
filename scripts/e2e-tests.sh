#!/bin/bash
set -e

# Navigate to project root
cd "$(dirname "$(realpath "$0")")/.."

export RPC_URL=http://localhost:8547
export DEPLOYER_ADDRESS=0x6ac4839Bfe169CadBBFbDE3f29bd8459037Bf64e

cargo run -p integration -- --rpc-url $RPC_URL --priv-key $DEPLOYER_ADDRESS

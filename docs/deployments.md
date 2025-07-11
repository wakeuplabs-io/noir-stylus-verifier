
# Deployments

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
just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc --set private_key ... deploy-contract sumcheck-verifier
# reading wasm file at ./target/wasm32-unknown-unknown/release/sumcheck-verifier-opt.wasm
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# contract size: 23.6 KiB (24132 bytes)
# wasm size: 79.6 KiB (81536 bytes)
# connecting to RPC: https://sepolia-rollup.arbitrum.io/rpc
# wasm data fee: 0.000121 ETH (originally 0.000101 ETH with 20% bump)
# sender address: 0x5cff4762b7a50553586d52f96c11aa65e9281d5a
# estimates
# deployment tx gas: 5314797
# gas price: "0.100000000" gwei
# deployment tx total cost: "0.000531479700000000" ETH
# sent deploy tx: 0x8d3e3dc0127f63757f9ef5ed3848a7d1483f8529039f06ff682fa93eeb9c3698
# deployed code at address: 0x64ede255c3e5b8f59fbfd014937ddd166b51705c with 5271784 gas
# deployment tx hash: 0x8d3e3dc0127f63757f9ef5ed3848a7d1483f8529039f06ff682fa93eeb9c3698
# activation gas estimate: 3575477 gas
# sent activate tx: 0x6c4ccb601ecc3f7ee4bf5b7cc51d7870d71469cf07e5a61869135f1d5614e15f
# activated with 3546166 gas
# contract activated and ready onchain with tx hash: 0x6c4ccb601ecc3f7ee4bf5b7cc51d7870d71469cf07e5a61869135f1d5614e15f

just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc --set private_key ... deploy-contract shplemini-verifier
# reading wasm file at ./target/wasm32-unknown-unknown/release/shplemini-verifier-opt.wasm
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# contract size: 23.5 KiB (24094 bytes)
# wasm size: 69.0 KiB (70698 bytes)
# connecting to RPC: https://sepolia-rollup.arbitrum.io/rpc
# wasm data fee: 0.000116 ETH (originally 0.000096 ETH with 20% bump)
# sender address: 0x5cff4762b7a50553586d52f96c11aa65e9281d5a
# estimates
# deployment tx gas: 5306286
# gas price: "0.100000000" gwei
# deployment tx total cost: "0.000530628600000000" ETH
# sent deploy tx: 0xb92224b211c1f4fe4a9cdf32584730c219e23c58c5abbdf31aecbc117ae4e721
# deployed code at address: 0x25eacbdf93a19a0b7ee705ad58f6ba3337cebffc with 5263340 gas
# deployment tx hash: 0xb92224b211c1f4fe4a9cdf32584730c219e23c58c5abbdf31aecbc117ae4e721
# activation gas estimate: 3432701 gas
# sent activate tx: 0xbd75e9c48fb0b56c56c8f7ac5b9f8024241956c4cc32318927b0d775a8896990
# activated with 3404514 gas
# contract activated and ready onchain with tx hash: 0xbd75e9c48fb0b56c56c8f7ac5b9f8024241956c4cc32318927b0d775a8896990


just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc --set private_key ... deploy-contract verifier "constructor(address,address)" "0x64ede255c3e5b8f59fbfd014937ddd166b51705c" "0x25eacbdf93a19a0b7ee705ad58f6ba3337cebffc"
# reading wasm file at ./target/wasm32-unknown-unknown/release/verifier-opt.wasm
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# contract size: 20.0 KiB (20512 bytes)
# wasm size: 59.2 KiB (60637 bytes)
# connecting to RPC: https://sepolia-rollup.arbitrum.io/rpc
# wasm data fee: 0.000114 ETH (originally 0.000095 ETH with 20% bump)
# sender address: 0x5cff4762b7a50553586d52f96c11aa65e9281d5a
# deploying contract using deployer at address: 0x6ac4839bfe169cadbbfbde3f29bd8459037bf64e
# estimates
# deployer deploy, activate, and init tx gas: 7866863
# gas price: "0.100000000" gwei
# deployer deploy, activate, and init tx total cost: "0.000786686300000000" ETH
# sent deploy_activate_init tx: 0x7011e5c0c4cb8eddda97de9625a23540eab13d21b28cb6cc7d7af2b269379c6d
# deployed code at address: 0x79693edb49473dc3522de16fbd047977c4999d5c with 7803737 gas
# deployment tx hash: 0x7011e5c0c4cb8eddda97de9625a23540eab13d21b28cb6cc7d7af2b269379c6d


just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc get-verifier-addresses "0x79693edb49473dc3522de16fbd047977c4999d5c"
just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc verify-proof "0x79693edb49473dc3522de16fbd047977c4999d5c" assert
```
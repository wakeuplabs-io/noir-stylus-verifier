
# Deployments

## UltraKeccakFlavor

Locally

```bash 
# Start the local devnet node
just nitro-testnode-up

# deploy
just deploy-contract sumcheck-verifier
just deploy-contract shplemini-verifier
just deploy-contract verifier "constructor(address,address)" "{sumcheck-verifier}" "{shplemini-verifier}"

# And verify with
just get-verifier-addresses "{verifier}"
just verify-proof "{verifier}" "{test_vector_name}"

# Once done, shut down the devnet node
just nitro-testnode-down
```

Sepolia

```bash
just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc --set private_key ... deploy-contract sumcheck-verifier
# reading wasm file at ./target/wasm32-unknown-unknown/release/sumcheck-verifier.wasm
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# contract size: 23.5 KiB (24016 bytes)
# wasm size: 90.8 KiB (92982 bytes)
# connecting to RPC: https://sepolia-rollup.arbitrum.io/rpc
# wasm data fee: 0.000150 ETH (originally 0.000125 ETH with 20% bump)
# sender address: 0x5cff4762b7a50553586d52f96c11aa65e9281d5a
# estimates
# deployment tx gas: 5287669
# gas price: "0.100000000" gwei
# deployment tx total cost: "0.000528766900000000" ETH
# sent deploy tx: 0x16ffdb9827180d609171895828afe4ac1db1e60cfecf02a874224e12a17d04e3
# deployed code at address: 0x96ca379c5cabd522c4f46586fab92e064a45063c with 5246675 gas
# deployment tx hash: 0x16ffdb9827180d609171895828afe4ac1db1e60cfecf02a874224e12a17d04e3
# activation gas estimate: 3878603 gas
# sent activate tx: 0x28c78fdffc91c16a3e55804dc74b91cab1fa26ef178dbf98f68fcc7a8d6fde89
# activated with 3848538 gas
# contract activated and ready onchain with tx hash: 0x28c78fdffc91c16a3e55804dc74b91cab1fa26ef178dbf98f68fcc7a8d6fde89

just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc --set private_key ... deploy-contract shplemini-verifier
# reading wasm file at ./target/wasm32-unknown-unknown/release/shplemini-verifier.wasm
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# contract size: 23.8 KiB (24381 bytes)
# wasm size: 78.6 KiB (80506 bytes)
# connecting to RPC: https://sepolia-rollup.arbitrum.io/rpc
# wasm data fee: 0.000144 ETH (originally 0.000120 ETH with 20% bump)
# sender address: 0x5cff4762b7a50553586d52f96c11aa65e9281d5a
# estimates
# deployment tx gas: 5367298
# gas price: "0.100000000" gwei
# deployment tx total cost: "0.000536729800000000" ETH
# sent deploy tx: 0x0e2986d615e9f783ac2e4eb972d98961014c0bef97f7074cd83fe2cd612e9e42
# deployed code at address: 0xf29e0d0d6992d3f8947bac144114f29b917d7d8e with 5325687 gas
# deployment tx hash: 0x0e2986d615e9f783ac2e4eb972d98961014c0bef97f7074cd83fe2cd612e9e42
# activation gas estimate: 3771993 gas
# sent activate tx: 0x58997174770295333c74916c2ee0ac99f9e3b7a178a5e7c8965115d1eb1b7abc
# activated with 3742754 gas
# contract activated and ready onchain with tx hash: 0x58997174770295333c74916c2ee0ac99f9e3b7a178a5e7c8965115d1eb1b7abc

just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc --set private_key ... deploy-contract verifier "constructor(address,address)" "0x96ca379c5cabd522c4f46586fab92e064a45063c" "0xf29e0d0d6992d3f8947bac144114f29b917d7d8e"
# reading wasm file at ./target/wasm32-unknown-unknown/release/verifier.wasm
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# contract size: 21.3 KiB (21838 bytes)
# wasm size: 68.3 KiB (69970 bytes)
# connecting to RPC: https://sepolia-rollup.arbitrum.io/rpc
# wasm data fee: 0.000145 ETH (originally 0.000121 ETH with 20% bump)
# sender address: 0x5cff4762b7a50553586d52f96c11aa65e9281d5a
# deploying contract using deployer at address: 0x6ac4839bfe169cadbbfbde3f29bd8459037bf64e
# estimates
# deployer deploy, activate, and init tx gas: 8510721
# gas price: "0.100000000" gwei
# deployer deploy, activate, and init tx total cost: "0.000851072100000000" ETH
# sent deploy_activate_init tx: 0xbd53c602d005bfdbd7b17ea8190de0b877ec236074757bbda0b49df77cd76ef1
# deployed code at address: 0x951d400a88f98c2d3f6f8af7b502a59bf418ab76 with 8444734 gas
# deployment tx hash: 0xbd53c602d005bfdbd7b17ea8190de0b877ec236074757bbda0b49df77cd76ef1


just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc verify-proof "0x951d400a88f98c2d3f6f8af7b502a59bf418ab76"

just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc get-verifier-addresses "0x951d400a88f98c2d3f6f8af7b502a59bf418ab76"
just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc verify-proof "0x951d400a88f98c2d3f6f8af7b502a59bf418ab76" assert
```

## UltraKeccakZKFlavor

Locally

```bash 
# Start the local devnet node
just nitro-testnode-up

# deploy
just deploy-contract sumcheck-verifier zk-flavored
just deploy-contract shplemini-verifier zk-flavored
just deploy-contract verifier zk-flavored "constructor(address,address)" "{sumcheck-verifier}" "{shplemini-verifier}"

# And verify with
just get-verifier-addresses "{verifier}"
just verify-proof "{verifier}" "{test_vector_name}" true

# Once done, shut down the devnet node
just nitro-testnode-down
```

Sepolia

```bash 
# Start the local devnet node
just nitro-testnode-up

# deploy
just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc --set private_key  ... deploy-contract zk-sumcheck-verifier
# reading wasm file at ./target/wasm32-unknown-unknown/release/sumcheck-verifier.wasm
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# contract size: 23.5 KiB (24019 bytes)
# wasm size: 90.8 KiB (92982 bytes)
# connecting to RPC: https://sepolia-rollup.arbitrum.io/rpc
# wasm data fee: 0.000150 ETH (originally 0.000125 ETH with 20% bump)
# sender address: 0x5cff4762b7a50553586d52f96c11aa65e9281d5a
# estimates
# deployment tx gas: 5288121
# gas price: "0.100000000" gwei
# deployment tx total cost: "0.000528812100000000" ETH
# sent deploy tx: 0x75feb797e628efe91011dc69d14bd628ea7d1e227bf07172ac7b40ea24657c42
# deployed code at address: 0x9602b2c5366db48b3615cadbdf1040d3c66e34fd with 5247114 gas
# deployment tx hash: 0x75feb797e628efe91011dc69d14bd628ea7d1e227bf07172ac7b40ea24657c42
# activation gas estimate: 3878604 gas
# sent activate tx: 0x66181beaa0ae659180fdd353deeafbe123bae8cda392bba19426524ea8f7ab9e
# activated with 3848538 gas
# contract activated and ready onchain with tx hash: 0x66181beaa0ae659180fdd353deeafbe123bae8cda392bba19426524ea8f7ab9e

just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc --set private_key  ... deploy-contract zk-shplemini-verifier
# reading wasm file at ./target/wasm32-unknown-unknown/release/shplemini-verifier.wasm
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# contract size: 23.9 KiB (24429 bytes)
# wasm size: 78.6 KiB (80506 bytes)
# connecting to RPC: https://sepolia-rollup.arbitrum.io/rpc
# wasm data fee: 0.000144 ETH (originally 0.000120 ETH with 20% bump)
# sender address: 0x5cff4762b7a50553586d52f96c11aa65e9281d5a
# estimates
# deployment tx gas: 5377677
# gas price: "0.100000000" gwei
# deployment tx total cost: "0.000537767700000000" ETH
# sent deploy tx: 0xdc5cd78c867ba05eb452de372b8d1f8a89a026e5b9999c891a5d6e4f07871923
# deployed code at address: 0x902142d9b95503a84e1c3535bd4ed410fcf063c8 with 5335975 gas
# deployment tx hash: 0xdc5cd78c867ba05eb452de372b8d1f8a89a026e5b9999c891a5d6e4f07871923
# activation gas estimate: 3771994 gas
# sent activate tx: 0x0d28017a05d9500f33f14649beddce4ed1998a17195fa5e24789135ce6d8ba4a
# activated with 3742754 gas
# contract activated and ready onchain with tx hash: 0x0d28017a05d9500f33f14649beddce4ed1998a17195fa5e24789135ce6d8ba4a

just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc --set private_key  ... deploy-contract zk-verifier "constructor(address,address)" 0x9602b2c5366db48b3615cadbdf1040d3c66e34fd 0x902142d9b95503a84e1c3535bd4ed410fcf063c8
# reading wasm file at ./target/wasm32-unknown-unknown/release/verifier.wasm
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# stripped custom section from user wasm to remove any sensitive data
# contract size: 23.9 KiB (24493 bytes)
# wasm size: 78.0 KiB (79887 bytes)
# connecting to RPC: https://sepolia-rollup.arbitrum.io/rpc
# sender address: 0x5cff4762b7a50553586d52f96c11aa65e9281d5a
# deploying contract using deployer at address: 0x6ac4839bfe169cadbbfbde3f29bd8459037bf64e
# estimates
# deployer deploy, activate, and init tx gas: 5502378
# gas price: "0.100000000" gwei
# deployer deploy, activate, and init tx total cost: "0.000550237800000000" ETH
# sent deploy_activate_init tx: 0x7a5fe0000a169555f0df2149a5a4b76c743bf76c17730867c3e5a124990d9725
# deployed code at address: 0xdcaaed24c926bc718984eaa4126e27b27d60379d with 5459688 gas
# deployment tx hash: 0x7a5fe0000a169555f0df2149a5a4b76c743bf76c17730867c3e5a124990d9725

# And verify with
just  --set rpc_url https://sepolia-rollup.arbitrum.io/rpc get-verifier-addresses 0xdcaaed24c926bc718984eaa4126e27b27d60379d
just  --set rpc_url https://sepolia-rollup.arbitrum.io/rpc verify-proof 0xdcaaed24c926bc718984eaa4126e27b27d60379d assert true
```

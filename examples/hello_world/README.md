
# Hello World example

Note: `@aztec/bb.js` specific version of `0.86.0` is important.


## Deployments

Locally

First ensure to have deployed the global verifier as per README.md instructions. Then  do:

```bash
just build-contract
just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc --set private_key ... deploy-contract "{{local verifier address}}"
```

Sepolia deployment

```bash
just build-contract
just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc --set private_key ... deploy-contract "0x79693edb49473dc3522de16fbd047977c4999d5c"

# stripped custom section from user wasm to remove any sensitive data
# contract size: 10.4 KiB (10667 bytes)
# wasm size: 28.3 KiB (28985 bytes)
# File used for deployment hash: ./Cargo.lock
# File used for deployment hash: ./Cargo.toml
# File used for deployment hash: ./rust-toolchain.toml
# File used for deployment hash: ./src/lib.rs
# File used for deployment hash: ./src/main.rs
# project metadata hash computed on deployment: "83b0ec1981477dd5889f98ce92b6250931e26419b51186285475b209d0f0dd64"
# reading wasm file at /Users/matzapata/git-work/aztec/noir-stylus-verifier/examples/hello_world/contracts/target/wasm32-unknown-unknown/release/deps/stylus_hello_world.wasm
# stripped custom section from user wasm to remove any sensitive data
# contract size: 10.4 KiB (10667 bytes)
# wasm size: 28.3 KiB (28985 bytes)
# connecting to RPC: https://sepolia-rollup.arbitrum.io/rpc
# checking whether the contract has a constructor...
# sender address: 0x5cff4762b7a50553586d52f96c11aa65e9281d5a
# deploying contract using deployer at address: 0x6ac4839bfe169cadbbfbde3f29bd8459037bf64e
# estimates
# deployer deploy, activate, and init tx gas: 2452550
# gas price: "0.100000000" gwei
# deployer deploy, activate, and init tx total cost: "0.000245255000000000" ETH
# sent deploy_activate_init tx: 0x927024373743a8b8116af65744b4143298874c00032ff1e38337675decb04552
# deployed code at address: 0x000f54dac67be18d85d601613360a8d3a0e835a8 with 2432061 gas
# deployment tx hash: 0x927024373743a8b8116af65744b4143298874c00032ff1e38337675decb04552

# verify with
just --set rpc_url https://sepolia-rollup.arbitrum.io/rpc verify "0x2ebfcd6f208943dcc0a5d07a41742ce00921469b"
```
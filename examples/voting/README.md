

## Deploy circuits

```bash
cd circuits
nsv generate
nsv check
nsv deploy --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key 0x...
```

Deployed address: `0xc2657f76cf69d2638471951e82a7a9aeab7c4bc4`

## Deploy contracts

Adjust constructor params with your circuit deployment address:

```bash
cd contracts
cargo stylus check
cargo stylus deploy --no-verify --endpoint https://sepolia-rollup.arbitrum.io/rpc --private-key "0x..." --constructor-args "0xc2657f76cf69d2638471951e82a7a9aeab7c4bc4"
```

Deployed address: `0xd48c45cd7255b80a3825e6714d444243609239be`
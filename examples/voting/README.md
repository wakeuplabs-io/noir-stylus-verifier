
## Deploy circuits

```bash
cd circuits
nsv generate
nsv check
nsv deploy --rpc-url https://sepolia-rollup.arbitrum.io/rpc  --private-key "0x..."
```

Deployed address: `0xb9f023155d199cec51b87dd5422d81518534563b`

## Deploy contracts

Adjust constructor params with your circuit deployment address:

```bash
cd contracts
cargo stylus check
cargo stylus deploy --no-verify --endpoint https://sepolia-rollup.arbitrum.io/rpc --private-key "0x..." --constructor-args "0xb9f023155d199cec51b87dd5422d81518534563b"
```

Deployed address: `0xe566251af974d9b71c99fbaf60368130b377cf90`

Update this address in `app/config/constants.ts`

## Example cli usage:

Create proposal: 

```bash
./app/cli.ts propose --voters voters.json --description "Demo" --deadline '2025-08-28T21:35:00.000Z' --private-key "0x..." --rpc-url https://sepolia-rollup.arbitrum.io/rpc
```

Check proposal status: 

```bash
./app/cli.ts get-proposal 1 --rpc-url https://sepolia-rollup.arbitrum.io/rpc
```

Vote:

```bash
./app/cli.ts cast-vote --proposal-id 0  --vote 1 --voters voters.json --private-key "0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" --rpc-url https://sepolia-rollup.arbitrum.io/rpc --relayer-private-key 0x...
```

Confirm your vote:

```bash
./app/cli.ts get-proposal 1 --rpc-url https://sepolia-rollup.arbitrum.io/rpc
```
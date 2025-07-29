
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

Deployed address: `0x67bd3931388eb74506e41beb48a53335d6fce92c`

Update this address in `app/config/constants.ts`

## Example cli usage:

From `apps/cli`, first create `.env` based on `.env.example.`. By default `private-key`, `relayer-private-key` and `rpc-url` are taken from there, you can also pass these as args.

Create a ZK account, by default it'll use your private key in `.env` remove it to create a new wallet as well or specify one as parameter:

```bash
./src/main.ts account
```

Include your ZK address in the proposal file! Make modifications as you please and when ready create the proposal with: 

```bash
./src/main.ts propose --proposal proposal.json 
```

Check proposal status: 

```bash
./src/main.ts get-proposal 1 
```

Vote:

```bash
./src/main.ts cast-vote --proposal-id 0  --vote 1 
```

Confirm your vote:

```bash
./src/main.ts get-proposal 1 --rpc-url https://sepolia-rollup.arbitrum.io/rpc
```
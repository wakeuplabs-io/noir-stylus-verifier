
# ZK Voting

> 🚨 Disclaimer: This repository is for educational purposes only. It is not audited, may contain bugs, and is not production-ready. Use at your own risk.

This example project shows how to develop a zero knowledge voting application using noir and arbitrum stylus, including the deployment of the noir verifier thought the noir stylus verifier cli.

We showcase 2 apps, the both are basically interfaces around the core package in `packages/core`. If you're interested in seeing the core domain check that out.  

There's a guide for this example at [nsv.wakeuplabs.link](https://nsv.wakeuplabs.link/docs/guides/building-a-voting-app). Check it out to know more about the project.

## How to run

Install workspace dependencies:

```bash
pnpm i
```

Both of these will require you to deploy the contracts, we already provide some arbitrum sepolia addresses so you can go with that if you want, otherwise here we show you how to deploy them.

For the circuits you need to have the `nsv` cli installed and ready to go then do:

```bash
cd circuits
nsv generate
nsv check
nsv deploy --rpc-url https://sepolia-rollup.arbitrum.io/rpc  --private-key "0x..."
```

Deployed address: `0xb9f023155d199cec51b87dd5422d81518534563b`


As for the contracts, adjust constructor params with your circuit deployment address:

```bash
cd contracts
cargo stylus check
cargo stylus deploy --no-verify --endpoint https://sepolia-rollup.arbitrum.io/rpc --private-key "0x..." --constructor-args "0xb9f023155d199cec51b87dd5422d81518534563b"
```

Deployed address: `0x67bd3931388eb74506e41beb48a53335d6fce92c`

Lastly create `.env` files from their example versions and fill with your values:
- CLI: `cp apps/cli/.env.example apps/cli/.env`
- WWW: `cp apps/www/.env.local.example apps/www/.env.local`


###  Running the web

Simply run with:

```bash
pnpm --filter=@voting/www dev
```

### Running the CLI 

You can run the cli by using `pnpm` from any part of the workspace like: `pnpm --filter=@voting/cli start ....` or by situating in `apps/cli`, adding permissions with `chmod +x ./src/main.ts` and then running: `./src/main.ts ....`


These are the available commands:

```bash
./src/main.ts --help

# Usage: cli [options] [command]

# Zero Knowledge Voting with Noir and Stylus

# Options:
#   -V, --version                         output the version number
#   -h, --help                            display help for command

# Commands:
#   account [options]                     Create a zk account from your evm private key
#   propose [options]                     Make a proposal for voting
#   get-proposal [options] <proposal-id>  Get a proposal
#   cast-vote [options]                   Cast a vote for a proposal
#   help [command]                        display help for command
```

`account`

```bash
./src/main.ts account --help

# Usage: cli account [options]

# Create a zk account from your evm private key

# Options:
#   -h, --help                       display help for command
```

`propose` (example proposal at `apps/cli/proposal.json`)

```bash
./src/main.ts propose --help

# Usage: cli propose [options]

# Make a proposal for voting

# Options:
#   --proposal <path to proposal>  File containing the proposal (default: "proposal.json")
#   -h, --help                     display help for command
```

`get-proposal`

```bash
./src/main.ts get-proposal --help

# Usage: cli get-proposal [options] <proposal-id>

# Get a proposal

# Options:
#   -h, --help           display help for command
```

`cast-vote`

```bash
./src/main.ts cast-vote --help

# Usage: cli cast-vote [options]

# Cast a vote for a proposal

# Options:
#   --proposal-id <proposal-id>                  Proposal ID
#   --vote <vote>                                Vote 1 in favor, 0 against
#   -h, --help                                   display help for command
```


**Example flow**


Create a ZK account, by default it'll use your private key in `.env`.

```bash
./src/main.ts account
```

Include your ZK address in the proposal file! Make modifications as you please and when ready create the proposal with: 

```bash
./src/main.ts propose --proposal proposal.json 
```

Check proposal status: 

```bash
./src/main.ts get-proposal 0
```

Vote:

```bash
./src/main.ts cast-vote --proposal-id 0  --vote 1 
```

Confirm your vote:

```bash
./src/main.ts get-proposal 0
```

## Credits

The design for the web app is heavily based on a [snapshot.box](https://snapshot.box/#/) and is used here for educational purposes only. All rights and credit go to the original website and its creators.

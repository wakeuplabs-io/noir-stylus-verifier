
# Battleships

> 🚨 Disclaimer: This repository is for educational purposes only. It is not audited, may contain bugs, and is not production-ready. Use at your own risk.

There's a guide for this example at [docs/tutorials/building-a-battleship-game](../../docs/tutorials/building-a-battleship-game.md). Check it out to know more about the project.

## Contracts

Deployments:

| Contract | Chain | Address |
|--------|-------|---------|
| BoardVerifier | Arbitrum Sepolia | 0xecb6faf4ade0e0a6df7b41ee9ba07c9cf5fdf205 |
| ShootVerifier | Arbitrum Sepolia | 0x62965b4f17523b61a295788d7fa6f269c940c5a3 |
| Battleship | Arbitrum Sepolia | 0xb3448a6f3958ac075182196dd717d5f574f81663 |

Deployed with: 

```bash
cd circuits/board
nsv generate
nsv check
nsv deploy --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key $DEV_PK

cd circuits/shoot
nsv generate
nsv check
nsv deploy --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key $DEV_PK 

cd contracts
cargo stylus deploy --no-verify --endpoint https://sepolia-rollup.arbitrum.io/rpc --private-key $DEV_PK --constructor-args 0xecb6faf4ade0e0a6df7b41ee9ba07c9cf5fdf205 0x62965b4f17523b61a295788d7fa6f269c940c5a3
```


## Web

Place yourself in `apps/www` and create `.env` based on `.env.example`. Then do: 

```bash
pnpm install
pnpm dev
```


## Cli

Place yourself in `apps/cli` and create `.env` based on `.env.example`. Recommend you to create `DEV_PK` and `DEV_PK_2` env variables for private keys of the players. Both must be funded with sepolia eth. Then of course: `pnpm install` and run with:

Player 1 (Join code must be unique onchain, so update it)

```bash
# create game
./src/main.ts create --private-key $DEV_PK  --join-code 123456

# play 
./src/main.ts play --private-key $DEV_PK 0x76d5d16d3eb5d7ba5349ed8364e09f3c256efb72d87d52df05f71d053ccd77e9
```

Player 2

```bash
# join game
./src/main.ts join --private-key $DEV_PK_2 --join-code 123456

# play 
./src/main.ts play --private-key $DEV_PK_2 0x76d5d16d3eb5d7ba5349ed8364e09f3c256efb72d87d52df05f71d053ccd77e9
```

## Credits

The design for the web app was sourced from the [Figma Community](https://www.figma.com/community/file/949373440973748315/battleship-game-interactive-components) and created by its respective author. All credit goes to the original designer.

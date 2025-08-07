
# Battleships

Deployments:

BoardVerifier: `0xecb6faf4ade0e0a6df7b41ee9ba07c9cf5fdf205`
ShootVerifier: `0x62965b4f17523b61a295788d7fa6f269c940c5a3`
Battleship: `0xb3448a6f3958ac075182196dd717d5f574f81663`

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

CLI usage

Player 1

```bash
# create game
./src/main.ts create --private-key $DEV_PK  --join-code 123456

# play 
./src/main.ts play --private-key $DEV_PK 0x76d5d16d3eb5d7ba5349ed8364e09f3c256efb72d87d52df05f71d053ccd77e9
./src/main.ts play --private-key $DEV_PK 0xb3926254d0163c91fc94bf08f3f8062ff5d48635dfa7dbef8168ad7d4847a758
```

Player 2

```bash
# join game
./src/main.ts join --private-key $DEV_PK_2 --join-code 123456
./src/main.ts join --private-key $DEV_PK_2 --join-code 321

# play 
./src/main.ts play --private-key $DEV_PK_2 0x76d5d16d3eb5d7ba5349ed8364e09f3c256efb72d87d52df05f71d053ccd77e9
./src/main.ts play --private-key $DEV_PK_2 0xb3926254d0163c91fc94bf08f3f8062ff5d48635dfa7dbef8168ad7d4847a758
./src/main.ts play --private-key $DEV_PK_2 0x8cdee82cb3ac6d59f1f417405a3eecf497b31f3d06d4c506f96deb67789f61e9
./src/main.ts play --private-key $DEV_PK_2 0x65e2d4887d2d88c1b77092432d3f62de56aa58bb187c0e4f79103401ef0c2fe1
```

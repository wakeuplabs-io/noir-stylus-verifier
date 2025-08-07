# Building ZK Battleship with Noir and Stylus

This tutorial demonstrates how to build an end-to-end zero-knowledge game using:
- **Noir** for writing ZK circuits
- **Stylus** for efficient smart contracts on Arbitrum
- **Noir Stylus Verifier** for generating Stylus-based proof verifiers

Zero-knowledge proofs allow you to prove a statement is true without revealing additional information. In blockchain gaming, this enables players to prove the validity of their moves without revealing private game state. zk-SNARKs are particularly well-suited for blockchain applications as they don't require interaction between the prover and verifier.

This example implements Battleship, a game where players must trust that opponents follow the rules without revealing their board configuration. The game requires:
1. Valid board setup
2. Immutable board state throughout the game
3. Private board configurations
4. Honest reporting of hits/misses

Source code: [noir-stylus-verifier/examples/battleship](https://github.com/wakeuplabs-io/noir-stylus-verifier/tree/main/examples/battleship)

## Architecture Overview

The ZK Battleship implementation uses zero-knowledge proofs to ensure game integrity while maintaining privacy. The system consists of:

**Noir Circuits:**
- Board validation circuit - proves valid ship placement
- Shot verification circuit - proves hit/miss results

**Stylus Contracts:**
- Game state management
- Proof verification via generated verifier contracts
- Turn-based gameplay logic

**Noir Stylus Verifier:**
- Automatically generates Stylus verifier contracts from Noir circuits
- Provides efficient on-chain proof verification

## ZK Circuit Design

The implementation requires two distinct circuits to handle different aspects of the game:

**1. Board Validation Circuit**
- **Private inputs**: Ship positions, nonce (salt)
- **Public inputs**: Board hash
- **Purpose**: Proves that a board configuration is valid (no overlapping ships, within bounds)

**2. Shot Verification Circuit**
- **Private inputs**: Ship positions, nonce
- **Public inputs**: Board hash, shot coordinates (x,y), hit result
- **Purpose**: Proves that a claimed hit/miss result is accurate for the given shot

## Implementation Stack

**Noir Circuits**
- [Noir](https://noir-lang.org/) - Domain-specific language for ZK circuits
- Rust-like syntax for easier development and maintenance
- Built-in cryptographic primitives (Poseidon hashing)

**Stylus Smart Contracts**
- [Arbitrum Stylus](https://docs.arbitrum.io/stylus) - Rust-based smart contracts
- WASM execution for gas efficiency
- Native Rust development experience

**Noir Stylus Verifier**
- Generates Stylus verifier contracts from Noir circuits
- Command-line tool: `nsv generate`, `nsv deploy`
- Seamless integration between Noir and Stylus

### Project Structure

```
examples/battleship/
├── circuits/
│   ├── common/          # Shared constants and utilities
│   ├── board/           # Board validation circuit
│   └── shoot/           # Shot validation circuit
├── contracts/           # Stylus smart contracts
├── apps/
│   ├── cli/            # Command-line interface
│   └── www/            # Web application
└── packages/
    └── core/           # Shared TypeScript library
```

## Noir Circuit Implementation

### Shared Constants and Utilities

The `circuits/common/src/lib.nr` module defines game constants and the board hashing function:

```noir
use poseidon::poseidon2::Poseidon2::hash;

pub global BOARD_SIZE: u32 = 10;
pub global NUMBER_OF_SHIPS: u32 = 5;

pub global SHIP_LENGTHS: [u8; NUMBER_OF_SHIPS] = [
    5, // Carrier
    4, // Battleship
    3, // Cruiser
    3, // Submarine
    2, // Destroyer
];

pub global X_INDEX: u32 = 0;
pub global Y_INDEX: u32 = 1;
pub global DIRECTION_INDEX: u32 = 2;
pub global SHIP_DIRECTION_DOWN: u8 = 0;
pub global SHIP_DIRECTION_RIGHT: u8 = 1;

pub fn hash_board(nonce: Field, ships: [[u8; 3]; NUMBER_OF_SHIPS]) -> Field {
    // Hash the board with the nonce.
    // Poseidon takes in a series of numbers, so we want to serialize each ship position as a number.
    // We know a Battleship position is (0...9), so we encode (x,y,p) array as a 3-digit number
    // ie, [3,2,1] would become "321"
    let mut hash_input: [Field; NUMBER_OF_SHIPS + 1] = [0; NUMBER_OF_SHIPS + 1];
    hash_input[0] = nonce;
    for i in 0..NUMBER_OF_SHIPS {
        let ship = ships[i];
        let x = ship[X_INDEX];
        let y = ship[Y_INDEX];
        let p = ship[DIRECTION_INDEX];
        hash_input[i + 1] = (x as Field) * 100 + (y as Field) * 10 + (p as Field);
    }

    let computed_board_hash = hash(hash_input, NUMBER_OF_SHIPS + 1);
    computed_board_hash
}
```

### Board Validation Circuit

The board validation circuit (`circuits/board/src/main.nr`) ensures valid ship placement:

```noir
use common::{
    EMPTY_BOARD, NUMBER_OF_SHIPS, SHIP_LENGTHS, SHIP_DIRECTION_DOWN,
    SHIP_DIRECTION_RIGHT, X_INDEX, Y_INDEX, BOARD_SIZE, DIRECTION_INDEX, hash_board,
};

// This circuit checks if the board is valid
// @param nonce: Field - Private salt for the hash
// @param ships: [[u8; 3]; NUMBER_OF_SHIPS] - [x,y,direction] for each ship
// @param board_hash: pub Field - Public hash of the board
fn main(nonce: Field, ships: [[u8; 3]; NUMBER_OF_SHIPS], board_hash: pub Field) {
    let mut board = EMPTY_BOARD;

    for i in 0..NUMBER_OF_SHIPS {
        let length = SHIP_LENGTHS[i];

        // Validate starting position is within bounds
        assert(ships[i][X_INDEX] as u32 >= 0 & ships[i][X_INDEX] as u32 < BOARD_SIZE);
        assert(ships[i][Y_INDEX] as u32 >= 0 & ships[i][Y_INDEX] as u32 < BOARD_SIZE);

        // Validate ships don't overflow off the board
        if (ships[i][DIRECTION_INDEX] == SHIP_DIRECTION_DOWN) {
            assert(ships[i][Y_INDEX] + length - 1 < BOARD_SIZE as u8);
        } else if (ships[i][DIRECTION_INDEX] == SHIP_DIRECTION_RIGHT) {
            assert(ships[i][X_INDEX] + length - 1 < BOARD_SIZE as u8);
        } else {
            assert(false, "Invalid direction");
        }

        // Validate no overlap between ships
        for l in 0..length {
            let x_offset = if ships[i][DIRECTION_INDEX] == SHIP_DIRECTION_DOWN { 0 } else { l };
            let y_offset = if ships[i][DIRECTION_INDEX] == SHIP_DIRECTION_DOWN { l } else { 0 };
            
            let x = ships[i][X_INDEX] + x_offset;
            let y = ships[i][Y_INDEX] + y_offset;
            
            assert(board[x as u32][y as u32] == 0);
            board[x as u32][y as u32] = 1;
        }
    }

    // Verify the provided hash matches our computed hash
    assert(board_hash == hash_board(nonce, ships));
}
```

**Key validations:**
- Ship placement within board boundaries
- No overlapping ships
- Hash verification against ship configuration
- Nonce-based protection against rainbow table attacks

### Shot Verification Circuit

The shot verification circuit (`circuits/shoot/src/main.nr`) validates hit/miss claims:

```noir
use common::{
    X_INDEX, Y_INDEX, BOARD_SIZE, DIRECTION_INDEX, NUMBER_OF_SHIPS,
    SHIP_DIRECTION_DOWN, SHIP_DIRECTION_RIGHT, SHIP_LENGTHS, hash_board,
};

// This circuit checks if a shot is a hit or a miss
fn main(
    nonce: Field,
    ships: [[u8; 3]; NUMBER_OF_SHIPS],
    board_hash: pub Field,
    x: pub u8,
    y: pub u8,
    hit: pub bool
) {
    // Validate the guess coordinates are within bounds
    assert(x >= 0 & x < BOARD_SIZE as u8);
    assert(y >= 0 & y < BOARD_SIZE as u8);

    // Validate the inputted ships match the public hash
    assert(board_hash == hash_board(nonce, ships));

    // Check if the shot hits any of the ships
    let hit0 = is_hit(x, y, ships[0], SHIP_LENGTHS[0]);
    let hit1 = is_hit(x, y, ships[1], SHIP_LENGTHS[1]);
    let hit2 = is_hit(x, y, ships[2], SHIP_LENGTHS[2]);
    let hit3 = is_hit(x, y, ships[3], SHIP_LENGTHS[3]);
    let hit4 = is_hit(x, y, ships[4], SHIP_LENGTHS[4]);
    
    assert(hit0 | hit1 | hit2 | hit3 | hit4 == hit);
}

fn is_hit(guess_x: u8, guess_y: u8, ship: [u8; 3], len: u8) -> bool {
    let mut is_hit = false;
    
    if (ship[DIRECTION_INDEX] == SHIP_DIRECTION_DOWN) {
        let x_match = guess_x == ship[X_INDEX];
        let y_in_range = (guess_y >= ship[Y_INDEX]) & (guess_y < ship[Y_INDEX] + len);
        is_hit = x_match & y_in_range;
    } else if (ship[DIRECTION_INDEX] == SHIP_DIRECTION_RIGHT) {
        let y_match = guess_y == ship[Y_INDEX];
        let x_in_range = (guess_x >= ship[X_INDEX]) & (guess_x < ship[X_INDEX] + len);
        is_hit = y_match & x_in_range;
    }
    
    is_hit
}
```

**Key validations:**
- Shot coordinates within board bounds
- Board integrity (ships unchanged since creation)
- Accurate hit/miss calculation
- Cryptographic proof of correctness

## Stylus Smart Contract

The game logic is implemented in Rust using the Stylus SDK. Key components include:

```rust
#[storage]
struct StorageGame {
    player1: StorageAddress,
    player2: StorageAddress,
    player1_board_hash: StorageU256,
    player2_board_hash: StorageU256,
    player1_points: StorageU256,
    player2_points: StorageU256,
    moves_count: StorageU256,
    moves: StorageMap<U256, StorageMove>,
}

#[public]
impl Battleship {
    /// Create a new game
    pub fn create_game(
        &mut self,
        game_id: U256,
        board_hash: U256,
        proof: Bytes,
    ) -> Result<(), BattleshipErrors> {
        // Verify the board is valid using the board verifier contract
        if !verify_board_proof(self.vm(), self.board_verifier.get(), proof, board_hash) {
            return Err(BattleshipErrors::InvalidProof(InvalidProof {}));
        }

        // Check game_id is not already taken
        if self.games.get(game_id).player1.get() != Address::ZERO {
            return Err(BattleshipErrors::GameAlreadyCreated(GameAlreadyCreated {}));
        }

        // Create the game
        let player1 = self.vm().msg_sender();
        let mut game = self.games.setter(game_id);
        game.player1.set(player1);
        game.player1_board_hash.set(board_hash);
        // ... initialize other fields

        Ok(())
    }

    /// Make a shot in the game
    pub fn shoot(
        &mut self,
        game_id: U256,
        previous_move_hit_proof: Bytes,
        previous_move_hit: bool,
        previous_move_x: U256,
        previous_move_y: U256,
        x: U256,
        y: U256,
    ) -> Result<(), BattleshipErrors> {
        // Validate shot coordinates
        if x >= U256::from(BOARD_SIZE) || y >= U256::from(BOARD_SIZE) {
            return Err(BattleshipErrors::InvalidShot(InvalidShot {}));
        }

        // For non-first moves, verify the previous move result with a proof
        if moves_count > U256::ZERO {
            if !verify_shoot_proof(
                vm,
                shoot_verifier_addr,
                previous_move_hit_proof,
                current_player_board_hash,
                previous_move_hit,
                previous_move_x,
                previous_move_y,
            ) {
                return Err(BattleshipErrors::InvalidProof(InvalidProof {}));
            }
        }

        // Record the new move and update game state
        // ...

        Ok(())
    }
}
```

**Core functionality:**
- Game state management (players, moves, scores)
- ZK proof verification via generated verifier contracts
- Turn-based game flow enforcement
- Win condition detection

## Noir Stylus Verifier Integration

The `noir-stylus-verifier` tool bridges Noir circuits and Stylus contracts:

### Circuit Compilation and Deployment

```bash
# Generate Stylus verifier from Noir circuit
cd circuits/board
nsv generate

# Deploy verifier contract
nsv deploy --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key $PRIVATE_KEY
```

**Process:**
1. Compiles Noir circuits to Stylus-compatible verifiers
2. Generates Rust verifier contracts
3. Deploys to Arbitrum with a single command
4. Returns contract addresses for integration

### Client-Side Proof Generation

Applications generate proofs using the Noir.js library and submit them to the Stylus contract for verification.

## User Interfaces

The example includes both CLI and web interfaces:

```bash
# Create a game
./src/main.ts create --private-key $PRIVATE_KEY --join-code 123456

# Join a game  
./src/main.ts join --private-key $PRIVATE_KEY_2 --join-code 123456

# Play the game
./src/main.ts play --private-key $PRIVATE_KEY $GAME_ID
```

## Live Deployment

The complete system is deployed on Arbitrum Sepolia:

| Contract | Address |
|----------|---------|
| BoardVerifier | `0xecb6faf4ade0e0a6df7b41ee9ba07c9cf5fdf205` |
| ShootVerifier | `0x62965b4f17523b61a295788d7fa6f269c940c5a3` |
| Battleship | `0xb3448a6f3958ac075182196dd717d5f574f81663` |

### Deployment Workflow

```bash
# 1. Generate and deploy verifier contracts
cd circuits/board
nsv generate && nsv deploy --rpc-url $RPC_URL --private-key $PRIVATE_KEY

cd ../shoot  
nsv generate && nsv deploy --rpc-url $RPC_URL --private-key $PRIVATE_KEY

# 2. Deploy main game contract with verifier addresses
cd ../../contracts
cargo stylus deploy --endpoint $RPC_URL --private-key $PRIVATE_KEY \
  --constructor-args <BOARD_VERIFIER_ADDR> <SHOOT_VERIFIER_ADDR>
```

## Key Benefits

**Zero-Knowledge Gaming**
- Trustless gameplay with hidden information
- Cryptographic guarantees instead of trusted third parties
- Privacy-preserving competitive gaming

**Noir Circuit Development**
- Rust-like syntax for familiar development experience
- Built-in cryptographic primitives
- Comprehensive testing framework

**Stylus Smart Contracts**
- Native Rust development for blockchain
- WASM execution efficiency
- Reduced gas costs compared to Solidity

**Noir Stylus Verifier Integration**
- Seamless workflow from circuit to verifier contract
- Single command deployment
- Type-safe proof verification

## Next Steps

This tutorial provides a foundation for building ZK applications with Noir and Stylus. Consider extending the implementation with:

- **Enhanced game features**: Multiple ship types, larger boards, power-ups
- **Optimized circuits**: Batch verification, recursive proofs
- **Advanced UI**: Real-time multiplayer, tournament modes
- **Cross-chain deployment**: Multi-chain game state synchronization

## Resources

- **Source Code**: [battleship example](https://github.com/wakeuplabs-io/noir-stylus-verifier/tree/main/examples/battleship)
- **Noir Documentation**: [noir-lang.org](https://noir-lang.org/)
- **Stylus Documentation**: [docs.arbitrum.io/stylus](https://docs.arbitrum.io/stylus)
- **NSV Tool**: [noir-stylus-verifier](https://github.com/wakeuplabs-io/noir-stylus-verifier)

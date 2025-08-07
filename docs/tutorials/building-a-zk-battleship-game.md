# BattleSnark: Building ZK Battleship with Noir and Stylus

I've been exploring zero-knowledge proofs recently, and took it as an opportunity to build out a fun project. This post walks through building a Battleship smart contract using zk-SNARKs, Noir circuits, and Stylus contracts on Arbitrum.

As a little background: Zero knowledge proofs let you prove a statement is true without revealing any additional information about the statement. In practice, this might look like showing you are credit worthy without handing over your bank statements, or validating nuclear disarmament without revealing state secrets. Typically, zero knowledge proofs involve repeatedly asking questions in order to validate the statement, but this is infeasible on the blockchain. zk-SNARK is a class of zero knowledge proofs that don't require interaction - which makes them a great fit for transactional use cases.

Thinking through the games I've played where we rely on other players not to cheat, Battleship stood out (who hasn't had an opponent move ships around during the game, or not even place all of their ships on the board!). The rules of battleship are pretty straightforward: Each player places 5 ships on their secret board (ensuring they are non-overlapping), and then players take turn guessing coordinates where their opponent's ships might be. The first person to guess all the coordinates of the other player's ships wins.

You can see the full source code at [noir-stylus-verifier/examples/battleship](https://github.com/wakeuplabs-io/noir-stylus-verifier/tree/main/examples/battleship).

## Battleship

Battleship involves trusting your opponent to set up a valid board, and record hits on their ships. Ideally there would be a way for a neutral 3rd party to validate the results without revealing any of the board state to the other player. The guarantees we need for a fair game are:

1. Each player's board is valid
2. Each player's board is unchanged throughout the game
3. A player shouldn't be able to see their opponent's board
4. A player can't lie about whether a guess is a hit or not

Building this in a centralized way is pretty straightforward: A "neutral" webserver/database can store the exact states secretly, and decide the results. As long as you trust the web host, you can trust the results aren't tampered with. Having this work on Ethereum is much more complicated though: Everything in the blockchain (smart contract data, transaction inputs, etc.) is public, so a player can't just send their ship locations in plain text for fear of their opponent seeing it. Let's see how zk-SNARK can help with this.

## zk-SNARK

A zk-SNARK is an arithmetic circuit which takes in a series of numerical signals and constraints, and derives a numerical output and a proof.

There are two types of inputs that make up the circuit:

1. **Public Inputs**: These are inputs that everyone knows (in Battleship, the guess coordinates are known to both parties so are public)
2. **Private Inputs**: These are inputs that only the provider knows (in Battleship, the locations of all the ships are private to only 1 player, so should be treated as private)

With a zk-SNARK, you can prove that a set of inputs generates an output, knowing only the public inputs and output. That seems to satisfy our criteria: We want a player to be able to have a private ship configuration, but still verifiably prove whether a public guess is a hit or not.

Based on our requirements above, we need two separate circuits:

1. **Board Creation**: One that can generate a hash identifier for a set of ship positions (and validate they are all properly positioned!)
2. **Move Validation**: One that can confirm/deny if a guess is a hit or not

## Getting Started with Noir

For this project, we're using [Noir](https://noir-lang.org/), a domain-specific language for writing zero-knowledge circuits. Unlike Circom, Noir provides a more developer-friendly syntax that resembles Rust, making it easier to write and maintain complex circuits.

### Project Structure

Our Battleship implementation has the following structure:

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

### Circuit 1: Board Creation

For board creation, we need one private input to represent the position of the ships, and we'll need an output that verifies the hash is based on the positions of the ships.

First, let's look at our common constants and utilities in `circuits/common/src/lib.nr`:

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

Now, the board validation circuit in `circuits/board/src/main.nr`:

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

This circuit does several important things:

1. **Validates ship placement**: Ensures all ships are within the board boundaries
2. **Prevents overlap**: Checks that no two ships occupy the same cell
3. **Verifies hash**: Confirms the provided hash matches the actual ship configuration
4. **Uses a nonce**: Prevents rainbow table attacks on ship configurations

### Circuit 2: Move Validation

The second circuit validates whether a shot is a hit or miss. Here's `circuits/shoot/src/main.nr`:

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

This circuit:

1. **Validates coordinates**: Ensures the shot is within the board
2. **Verifies board integrity**: Confirms the ships haven't changed since board creation
3. **Calculates hit/miss**: Determines if the shot hits any ship
4. **Proves correctness**: Generates a proof that the hit/miss claim is accurate

## Smart Contract Implementation

Our smart contract is written in Rust using the [Stylus SDK](https://docs.arbitrum.io/stylus/stylus-gentle-introduction) and deployed on Arbitrum. Here are the key parts of `contracts/src/lib.rs`:

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

The smart contract:

1. **Stores game state**: Tracks players, board hashes, moves, and scores
2. **Verifies proofs**: Calls the verifier contracts to validate ZK proofs
3. **Enforces game rules**: Ensures proper turn order and valid moves
4. **Manages game lifecycle**: Handles game creation, joining, and completion

## Proof Generation and Verification

### Generating Proofs

The proof generation happens client-side using the compiled Noir circuits. The `noir-stylus-verifier` tool automatically:

1. Compiles Noir circuits to verification contracts
2. Deploys them on Arbitrum
3. Provides a simple interface for proof verification

### Building the User Interface

We built both a CLI tool and web application for interacting with the game:

```bash
# Create a game
./src/main.ts create --private-key $PRIVATE_KEY --join-code 123456

# Join a game  
./src/main.ts join --private-key $PRIVATE_KEY_2 --join-code 123456

# Play the game
./src/main.ts play --private-key $PRIVATE_KEY $GAME_ID
```

## Deployment

The system is deployed on Arbitrum Sepolia:

| Contract | Address |
|----------|---------|
| BoardVerifier | `0xecb6faf4ade0e0a6df7b41ee9ba07c9cf5fdf205` |
| ShootVerifier | `0x62965b4f17523b61a295788d7fa6f269c940c5a3` |
| Battleship | `0xb3448a6f3958ac075182196dd717d5f574f81663` |

Deployment process:

```bash
# Generate and deploy verifier contracts
cd circuits/board && nsv generate && nsv deploy --rpc-url $RPC_URL --private-key $PRIVATE_KEY
cd circuits/shoot && nsv generate && nsv deploy --rpc-url $RPC_URL --private-key $PRIVATE_KEY

# Deploy main contract
cd contracts && cargo stylus deploy --endpoint $RPC_URL --private-key $PRIVATE_KEY
```

## Key Innovations

This implementation showcases several important concepts:

1. **Zero-Knowledge Gaming**: Demonstrates how ZK proofs enable trustless gaming with hidden information
2. **Noir Integration**: Shows practical use of Noir for circuit development
3. **Stylus Contracts**: Leverages Rust smart contracts on Arbitrum for efficiency
4. **Noir In Stylus**: Take full advantage of stylus capabilities with [nsv](https://github.com/wakeuplabs-io/noir-stylus-verifier) by generating stylus verifiers for your circuits.
4. **Modular Architecture**: Clean separation between circuits, contracts, and applications

## Conclusion

Building ZK Battleship demonstrates the power of zero-knowledge proofs for creating trustless applications with private state. The combination of Noir for circuit development and Stylus for smart contracts provides a powerful toolkit for building next-generation decentralized applications.

The key insight is that zero-knowledge proofs allow us to maintain the integrity of game rules while preserving the privacy that makes games like Battleship interesting. Players can prove their moves are valid without revealing their board configuration, creating a truly fair and trustless gaming experience.

Whether you're building games, privacy-preserving applications, or exploring ZK technology, this project provides a practical example of how these cutting-edge cryptographic tools can be applied to real-world problems.

You can explore the full codebase and try the game yourself at the [noir-stylus-verifier repository](https://github.com/wakeuplabs-io/noir-stylus-verifier/tree/main/examples/battleship).

---

*This tutorial was created as part of the noir-stylus-verifier project, demonstrating how to build zero-knowledge applications on Arbitrum using Noir and Stylus.*

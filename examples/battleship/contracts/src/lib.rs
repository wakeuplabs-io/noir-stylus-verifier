// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::Address;
use alloy_sol_types::sol;
use alloy_sol_types::SolCall;
use alloy_sol_types::SolType;
use stylus_sdk::crypto::keccak;
use stylus_sdk::storage::StorageBool;
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::U256,
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageU256},
    stylus_core::calls::context::Call,
};

sol! {
    // events
    event GameCreated(uint256 gameId, address player);
    event GameJoined(uint256 gameId, address player);
    event MoveMade(uint256 gameId, address player, uint256 x, uint256 y);
    event GameFinished(uint256 gameId, address winner);

    // errors
    #[derive(Debug)]
    error GameAlreadyCreated();
    #[derive(Debug)]
    error GameAlreadyJoined();
    #[derive(Debug)]
    error GameNotFound();
    #[derive(Debug)]
    error GameNotStarted();
    #[derive(Debug)]
    error GameNotYourTurn();
    #[derive(Debug)]
    error GameNotYourBoard();
    #[derive(Debug)]
    error GameNotYourMove();
    #[derive(Debug)]
    error GameNotYourShot();
    #[derive(Debug)]
    error InvalidShot();
    #[derive(Debug)]
    error InvalidProof();
    #[derive(Debug)]
    error InvalidJoinCode();
    #[derive(Debug)]
    error GameAlreadyFinished();

    // verify prototype
    function verify(bytes memory proof, bytes memory input) external view returns (bool);
}

#[storage]
struct StorageMove {
    x: StorageU256,
    y: StorageU256,
    is_hit: StorageBool, // 0 = false, 1 = true
}

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

#[derive(Debug, SolidityError)]
pub enum BattleshipErrors {
    GameAlreadyCreated(GameAlreadyCreated),
    GameAlreadyJoined(GameAlreadyJoined),
    GameAlreadyFinished(GameAlreadyFinished),
    GameNotFound(GameNotFound),
    GameNotStarted(GameNotStarted),
    GameNotYourTurn(GameNotYourTurn),
    GameNotYourBoard(GameNotYourBoard),
    GameNotYourMove(GameNotYourMove),
    GameNotYourShot(GameNotYourShot),
    InvalidShot(InvalidShot),
    InvalidProof(InvalidProof),
    InvalidJoinCode(InvalidJoinCode),
}

const BOARD_SIZE: u8 = 10;
const MAX_POINTS: u8 = 17;

#[storage]
#[entrypoint]
struct Battleship {
    board_verifier: StorageAddress,
    shoot_verifier: StorageAddress,
    games: StorageMap<U256, StorageGame>,
}

#[public]
impl Battleship {
    #[constructor]
    pub fn constructor(&mut self, board_verifier: Address, shoot_verifier: Address) {
        self.board_verifier.set(board_verifier);
        self.shoot_verifier.set(shoot_verifier);
    }

    /// Get the verifier address
    /// @return The verifier address
    pub fn get_board_verifier(&self) -> Address {
        self.board_verifier.get()
    }

    /// Get the verifier address
    /// @return The verifier address
    pub fn get_shot_verifier(&self) -> Address {
        self.shoot_verifier.get()
    }

    /// Create a new game
    /// @param game_id User hash of the join code
    /// @param board_hash The hash of the board
    /// @param proof The proof that the board is valid
    pub fn create_game(
        &mut self,
        game_id: U256,
        board_hash: U256,
        proof: Bytes,
    ) -> Result<(), BattleshipErrors> {
        // verify the board is valid
        if !verify_board_proof(self.vm(), self.board_verifier.get(), proof, board_hash) {
            return Err(BattleshipErrors::InvalidProof(InvalidProof {}));
        }

        // check game_id is not already taken
        if self.games.get(game_id).player1.get() != Address::ZERO {
            return Err(BattleshipErrors::GameAlreadyCreated(GameAlreadyCreated {}));
        }

        // create the game
        let player1 = self.vm().msg_sender();
        let mut game = self.games.setter(game_id);
        game.player1.set(player1);
        game.player2.set(Address::ZERO);
        game.player1_board_hash.set(board_hash);
        game.player2_board_hash.set(U256::ZERO);
        game.moves_count.set(U256::ZERO);
        game.player1_points.set(U256::ZERO);
        game.player2_points.set(U256::ZERO);
        // moves map will be empty by default

        log(
            self.vm(),
            GameCreated {
                gameId: game_id,
                player: player1,
            },
        );

        Ok(())
    }

    /// Join a game
    /// @param game_id The id of the game
    /// @param proof The proof that the board is valid
    /// @param board_hash The hash of the board
    /// @param join_code The join code for the game
    pub fn join_game(
        &mut self,
        proof: Bytes,
        board_hash: U256,
        join_code: U256,
    ) -> Result<(), BattleshipErrors> {
        let vm = self.vm();
        let board_verifier_addr = self.board_verifier.get();

        let game_id = keccak(join_code.to_be_bytes::<32>()).into();

        // check game exists
        let game = self.games.get(game_id);
        if game.player2.get() != Address::ZERO {
            return Err(BattleshipErrors::GameAlreadyJoined(GameAlreadyJoined {}));
        } else if game.player1.get() == vm.msg_sender() {
            return Err(BattleshipErrors::GameAlreadyJoined(GameAlreadyJoined {}));
        }

        // verify the board is valid
        if !verify_board_proof(vm, board_verifier_addr, proof, board_hash) {
            return Err(BattleshipErrors::InvalidProof(InvalidProof {}));
        }

        // store the player2 address and board hash
        let current_player = vm.msg_sender();
        let mut game = self.games.setter(game_id);
        game.player2.set(current_player);
        game.player2_board_hash.set(board_hash);

        Ok(())
    }

    /// Shoot at a game
    /// @param game_id The id of the game
    /// @param previous_move_hit_proof The proof that checks the previous move was a hit
    /// @param previous_move_hit Whether the previous move was a hit
    /// @param previous_move_x The x coordinate of the previous move
    /// @param previous_move_y The y coordinate of the previous move
    /// @param x The x coordinate of the new shot
    /// @param y The y coordinate of the new shot
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
        let vm = self.vm();
        let shoot_verifier_addr = self.shoot_verifier.get();
        let current_player = vm.msg_sender();

        // check shot is within range
        if x >= U256::from(BOARD_SIZE) || y >= U256::from(BOARD_SIZE) {
            return Err(BattleshipErrors::InvalidShot(InvalidShot {}));
        }

        // check game exists and theres's no winner yet
        let game = self.games.get(game_id);
        let player_1 = game.player1.get();
        let player_2 = game.player2.get();
        let mut player_1_points = game.player1_points.get();
        let mut player_2_points = game.player2_points.get();
        if player_1 == Address::ZERO || player_2 == Address::ZERO {
            return Err(BattleshipErrors::GameNotFound(GameNotFound {}));
        } else if player_1_points == U256::from(10) || player_2_points == U256::from(10) {
            return Err(BattleshipErrors::GameAlreadyFinished(GameAlreadyFinished {}));
        }

        // get game state
        let moves_count = game.moves_count.get();
        let current_player_board_hash = if player_1 == current_player {
            game.player1_board_hash.get()
        } else {
            game.player2_board_hash.get()
        };

        // verify the shot is valid, first move doesn't need a proof
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

        // update the game state
        let mut game = self.games.setter(game_id);
        if moves_count > U256::ZERO {
            let mut previous_move = game.moves.setter(moves_count - U256::ONE);
            previous_move.is_hit.set(previous_move_hit);
        }

        // add the new move
        let mut new_move = game.moves.setter(moves_count);
        new_move.x.set(x);
        new_move.y.set(y);
        new_move.is_hit.set(false); // Will be set on the next move

        game.moves_count.set(moves_count + U256::ONE);



        // update points if the previous move was a hit
        if previous_move_hit {
            if player_1 == current_player {
                player_1_points = player_1_points + U256::ONE;
                game.player1_points.set(player_1_points);
            } else {
                player_2_points = player_2_points + U256::ONE;
                game.player2_points.set(player_2_points);
            }

            // check  if the game finished
            if player_1_points == U256::from(MAX_POINTS) || player_2_points == U256::from(MAX_POINTS) {
                let winner = if player_1_points == U256::from(MAX_POINTS) {
                    game.player1.get()
                } else {
                    game.player2.get()
                };

                log(
                    self.vm(),
                    GameFinished {
                        gameId: game_id,
                        winner: winner,
                    },
                );
            }
        }


        log(
            self.vm(),
            MoveMade {
                gameId: game_id,
                player: current_player,
                x: x,
                y: y,
            },
        );

        Ok(())
    }

    /// Get the players of a game
    /// @param game_id The id of the game
    /// @return The players of the game
    pub fn get_game_players(&self, game_id: U256) -> Result<(Address, Address), BattleshipErrors> {
        let game = self.games.get(game_id);
        if game.player1.get() == Address::ZERO {
            return Err(BattleshipErrors::GameNotFound(GameNotFound {}));
        }
        Ok((game.player1.get(), game.player2.get()))
    }

    /// Get the boards hashes of a game
    /// @param game_id The id of the game
    /// @return The boards hashes of the game
    pub fn get_game_boards_hashes(&self, game_id: U256) -> Result<(U256, U256), BattleshipErrors> {
        let game = self.games.get(game_id);
        Ok((game.player1_board_hash.get(), game.player2_board_hash.get()))
    }

    /// Get the number of moves of a game
    pub fn get_game_move_count(&self, game_id: U256) -> Result<U256, BattleshipErrors> {
        let game = self.games.get(game_id);
        Ok(game.moves_count.get())
    }

    /// Get a move of a game
    /// @param game_id The id of the game
    /// @param move_index The index of the move
    /// @return The move x, y and is_hit
    pub fn get_game_move(
        &self,
        game_id: U256,
        move_index: U256,
    ) -> Result<(U256, U256, bool), BattleshipErrors> {
        let game = self.games.get(game_id);
        let game_move = game.moves.get(move_index);
        Ok((game_move.x.get(), game_move.y.get(), game_move.is_hit.get()))
    }
}

fn verify_board_proof(
    vm: &dyn Host,
    board_verifier_addr: Address,
    proof: Bytes,
    board_hash: U256,
) -> bool {
    static_call_helper::<verifyCall>(
        vm,
        board_verifier_addr,
        (proof.to_vec().into(), board_hash.to_be_bytes::<32>().into()),
    )
    .map(|res| res._0)
    .unwrap_or(false)
}

fn verify_shoot_proof(
    vm: &dyn Host,
    shoot_verifier_addr: Address,
    proof: Bytes,
    board_hash: U256,
    hit: bool,
    x: U256,
    y: U256,
) -> bool {
    static_call_helper::<verifyCall>(
        vm,
        shoot_verifier_addr,
        (
            proof.to_vec().into(),
            Vec::from(
                [
                    board_hash.to_be_bytes::<32>(),
                    x.to_be_bytes::<32>(),
                    y.to_be_bytes::<32>(),
                    if hit { U256::from(1) } else { U256::from(0) }.to_be_bytes::<32>(),
                ]
                .concat(),
            )
            .into(),
        ),
    )
    .map(|res| res._0)
    .unwrap_or(false)
}

fn static_call_helper<C: SolCall>(
    vm: &dyn Host,
    address: Address,
    args: <C::Parameters<'_> as SolType>::RustType,
) -> Result<C::Return, Vec<u8>> {
    let calldata = C::new(args).abi_encode();
    let res = vm
        .static_call(&Call::new(), address, &calldata)
        .map_err(|_| b"Call failed".to_vec())?;
    C::abi_decode_returns(&res, false).map_err(|_| b"Failed to decode return data".to_vec())
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::address;
    use alloy_sol_types::SolEvent;
    use stylus_sdk::crypto::keccak;
    use stylus_sdk::testing::*;

    sol! {
        function verify(bytes memory proof, bytes memory input) external view returns (bool);
    }

    const MOCK_JOIN_CODE: u32 = 12345;
    const MOCK_BOARD_HASH: u32 = 1000;
    const MOCK_PROOF: &[u8] = b"proof";
    const MOCK_BOARD_VERIFIER: Address = address!("0x0000000000000000000000000000000000000001");
    const MOCK_SHOT_VERIFIER: Address = address!("0x0000000000000000000000000000000000000002");

    fn mock_valid_board_proof(
        vm: &TestVM,
        contract: &mut Battleship,
        proof: &[u8],
        board_hash: u32,
    ) {
        // build calldata for mock
        let calldata = verifyCall {
            proof: proof.to_vec().into(),
            input: Vec::from([U256::from(board_hash).to_be_bytes::<32>()].concat()).into(),
        }
        .abi_encode();
        let return_data = verifyCall::abi_encode_returns(&(true,));
        vm.mock_static_call(contract.board_verifier.get(), calldata, Ok(return_data));
    }

    #[test]
    fn test_create_game() {
        let vm = TestVM::default();
        let mut contract = Battleship::from(&vm);
        contract.constructor(MOCK_BOARD_VERIFIER, MOCK_SHOT_VERIFIER);

        // mock valid board proof
        mock_valid_board_proof(&vm, &mut contract, MOCK_PROOF, MOCK_BOARD_HASH);

        let game_id = keccak(U256::from(MOCK_JOIN_CODE).to_be_bytes::<32>()).into();

        // create game
        contract
            .create_game(
                game_id,
                U256::from(MOCK_BOARD_HASH),
                Bytes::from(MOCK_PROOF.to_vec()),
            )
            .unwrap();

        // check we emitted event for the game creation
        let events = vm.get_emitted_logs();
        assert_eq!(events.len(), 1);

        // check the specific event content
        let (topics, data) = &events[0];

        // Check event signature (first topic)
        let expected_signature = GameCreated::SIGNATURE_HASH;
        assert_eq!(topics[0].as_slice(), expected_signature.as_slice());

        // Decode the event data
        let (game_id, creator) = GameCreated::abi_decode_data(data, false).unwrap();
        assert_eq!(game_id, game_id);
        assert_eq!(creator, vm.msg_sender());
    }
}

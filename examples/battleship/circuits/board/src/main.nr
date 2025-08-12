use common::{
    BOARD_SIZE, DIRECTION_INDEX, EMPTY_BOARD, hash_board, NUMBER_OF_SHIPS, SHIP_DIRECTION_DOWN,
    SHIP_DIRECTION_RIGHT, SHIP_LENGTHS, X_INDEX, Y_INDEX,
};

// This circuit checks if the board is valid
// @param nonce: Field
// @param ships: [x,y,direction:0=down,1=right]. These are ordered Carrier, Battleship, Cruiser, Submarine, Destroyer.
fn main(nonce: Field, ships: [[u8; 3]; NUMBER_OF_SHIPS], board_hash: pub Field) {
    let mut board = EMPTY_BOARD;

    for i in 0..NUMBER_OF_SHIPS {
        let length = SHIP_LENGTHS[i];

        // validate starting position
        assert(ships[i][X_INDEX] as u32 >= 0 & ships[i][X_INDEX] as u32 < BOARD_SIZE);
        assert(ships[i][Y_INDEX] as u32 >= 0 & ships[i][Y_INDEX] as u32 < BOARD_SIZE);

        // validate boats don't overflow off board
        if (ships[i][DIRECTION_INDEX] == SHIP_DIRECTION_DOWN) {
            assert(ships[i][Y_INDEX] + length - 1 < BOARD_SIZE as u8);
        } else if (ships[i][DIRECTION_INDEX] == SHIP_DIRECTION_RIGHT) {
            assert(ships[i][X_INDEX] + length - 1 < BOARD_SIZE as u8);
        } else {
            assert(false, "Invalid direction");
        }

        // validate no overlap
        for l in 0..length {
            let x_a = if ships[i][DIRECTION_INDEX] == SHIP_DIRECTION_DOWN {
                0
            } else {
                l
            };
            let y_a = if ships[i][DIRECTION_INDEX] == SHIP_DIRECTION_DOWN {
                l
            } else {
                0
            };
            let x = ships[i][X_INDEX] + x_a;
            let y = ships[i][Y_INDEX] + y_a;
            assert(board[x as u32][y as u32] == 0);
            board[x as u32][y as u32] = 1;
        }
    }

    assert(board_hash == hash_board(nonce, ships));
}

#[test]
fn test_hash_input() {
    let nonce = 12345;
    let ships = [[2, 2, 0], [4, 0, 1], [1, 0, 0], [5, 5, 1], [6, 3, 0]];
    let board_hash = 0x0cf4157c9c686528efc9b5c573b4b80538780af478f0b41e3f7d69749231c352;

    main(nonce, ships, board_hash);
}

#[test]
fn test_valid_edge_placement() {
    let nonce = 9999;
    // Place first ship horizontally touching the right edge exactly, others simple non-overlapping
    let x0 = BOARD_SIZE as u8 - SHIP_LENGTHS[0];
    let ships = [
        [x0, 0, SHIP_DIRECTION_RIGHT],
        [0, 2, SHIP_DIRECTION_DOWN],
        [3, 3, SHIP_DIRECTION_RIGHT],
        [6, 1, SHIP_DIRECTION_DOWN],
        [8, 8, SHIP_DIRECTION_RIGHT],
    ];
    let board_hash = hash_board(nonce, ships);
    main(nonce, ships, board_hash);
}

#[test(should_fail)]
fn test_hash_mismatch_should_fail() {
    let nonce = 12345;
    let ships = [[2, 2, 0], [4, 0, 1], [1, 0, 0], [5, 5, 1], [6, 3, 0]];
    // Intentionally wrong public hash
    let wrong_board_hash = 0;
    main(nonce, ships, wrong_board_hash);
}

#[test(should_fail_with = "Invalid direction")]
fn test_invalid_direction_should_fail() {
    let nonce = 12345;
    // First ship has an invalid direction (neither down nor right)
    let ships = [[2, 2, 2], [4, 0, 1], [1, 0, 0], [5, 5, 1], [6, 3, 0]];
    let board_hash = hash_board(nonce, ships);
    main(nonce, ships, board_hash);
}

#[test(should_fail)]
fn test_overflow_off_board_should_fail() {
    let nonce = 12345;
    // Make the first ship vertical starting too close to the bottom so it overflows
    let y0 = BOARD_SIZE as u8 - SHIP_LENGTHS[0] + 1; // one past the allowed start
    let ships = [
        [0, y0, SHIP_DIRECTION_DOWN],
        [4, 0, SHIP_DIRECTION_RIGHT],
        [1, 0, SHIP_DIRECTION_DOWN],
        [5, 5, SHIP_DIRECTION_RIGHT],
        [6, 3, SHIP_DIRECTION_DOWN],
    ];
    let board_hash = hash_board(nonce, ships);
    main(nonce, ships, board_hash);
}

#[test(should_fail)]
fn test_overlapping_ships_should_fail() {
    let nonce = 12345;
    // Two ships start at the same position and overlap
    let ships = [
        [0, 0, SHIP_DIRECTION_RIGHT],
        [0, 0, SHIP_DIRECTION_DOWN],
        [3, 3, SHIP_DIRECTION_RIGHT],
        [5, 5, SHIP_DIRECTION_RIGHT],
        [6, 3, SHIP_DIRECTION_DOWN],
    ];
    let board_hash = hash_board(nonce, ships);
    main(nonce, ships, board_hash);
}

#[test(should_fail)]
fn test_start_position_out_of_bounds_should_fail() {
    let nonce = 12345;
    // First ship starts with x == BOARD_SIZE (invalid, equal to upper bound)
    let ships = [
        [BOARD_SIZE as u8, 0, SHIP_DIRECTION_RIGHT],
        [0, 2, SHIP_DIRECTION_DOWN],
        [3, 3, SHIP_DIRECTION_RIGHT],
        [6, 1, SHIP_DIRECTION_DOWN],
        [8, 8, SHIP_DIRECTION_RIGHT],
    ];
    let board_hash = hash_board(nonce, ships);
    main(nonce, ships, board_hash);
}

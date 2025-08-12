use common::{
    X_INDEX, Y_INDEX, BOARD_SIZE, DIRECTION_INDEX, NUMBER_OF_SHIPS, SHIP_DIRECTION_DOWN,
    SHIP_DIRECTION_RIGHT, SHIP_LENGTHS, hash_board, 
};

// This circuit checks if a shot is a hit or a miss
// @param nonce: Field
// @param ships: [x,y,direction:0=down,1=right]. These are ordered Carrier, Battleship, Cruiser, Submarine, Destroyer.
// @param board_hash: pub Field
// @param x: pub u8
// @param y: pub u8
// @param hit: pub bool
fn main(nonce: Field, ships: [[u8; 3]; NUMBER_OF_SHIPS], board_hash: pub Field, x: pub u8, y: pub u8, hit: pub bool) {
    // validate the guess is actually valid
    assert(x >= 0 & x < BOARD_SIZE as u8);
    assert(y >= 0 & y < BOARD_SIZE as u8);

    // validate the inputted ships matches the public hash
    assert(board_hash == hash_board(nonce, ships));

    // check if it's a hit to any of the ships
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
        let y_match = guess_y == ship[1];
        let x_in_range = (guess_x >= ship[X_INDEX]) & (guess_x < ship[X_INDEX] + len);
        is_hit = y_match & x_in_range;
    } else {
        assert(false, "Invalid direction");
    }

    is_hit
}

#[test]
fn test_hit() {
    let nonce = 12345;
    let ships = [[2, 2, 0], [4, 0, 1], [1, 0, 0], [5, 5, 1], [6, 3, 0]];
    let board_hash = 0x0cf4157c9c686528efc9b5c573b4b80538780af478f0b41e3f7d69749231c352;
    let x = 1;
    let y = 2;

    main(nonce, ships, board_hash, x, y, true);
}

#[test]
fn test_miss() {
    let nonce = 12345;
    let ships = [[2, 2, 0], [4, 0, 1], [1, 0, 0], [5, 5, 1], [6, 3, 0]];
    let board_hash = 0x0cf4157c9c686528efc9b5c573b4b80538780af478f0b41e3f7d69749231c352;
    let x = 1;
    let y = 3;

    main(nonce, ships, board_hash, x, y, false);
}

#[test]
fn test_edge_hit_right_boundary() {
    let nonce = 9999;
    // First ship touches the right edge exactly
    let x0 = BOARD_SIZE as u8 - SHIP_LENGTHS[0];
    let ships = [
        [x0, 0, SHIP_DIRECTION_RIGHT],
        [0, 2, SHIP_DIRECTION_DOWN],
        [3, 3, SHIP_DIRECTION_RIGHT],
        [6, 1, SHIP_DIRECTION_DOWN],
        [1, BOARD_SIZE as u8 - 1, SHIP_DIRECTION_RIGHT],
    ];
    let board_hash = hash_board(nonce, ships);

    // Guess the last cell on the right boundary for the first ship
    let x = BOARD_SIZE as u8 - 1;
    let y = 0;
    main(nonce, ships, board_hash, x, y, true);
}

#[test]
fn test_edge_miss_bottom_left() {
    let nonce = 9999;
    let x0 = BOARD_SIZE as u8 - SHIP_LENGTHS[0];
    let ships = [
        [x0, 0, SHIP_DIRECTION_RIGHT],
        [0, 2, SHIP_DIRECTION_DOWN],
        [3, 3, SHIP_DIRECTION_RIGHT],
        [6, 1, SHIP_DIRECTION_DOWN],
        [1, BOARD_SIZE as u8 - 1, SHIP_DIRECTION_RIGHT],
    ];
    let board_hash = hash_board(nonce, ships);

    // Bottom-left corner should be empty with the chosen layout
    let x = 0;
    let y = BOARD_SIZE as u8 - 1;
    main(nonce, ships, board_hash, x, y, false);
}

#[test(should_fail)]
fn test_out_of_bounds_x_should_fail() {
    let nonce = 1111;
    let ships = [[2, 2, 0], [4, 0, 1], [1, 0, 0], [5, 5, 1], [6, 3, 0]];
    let board_hash = hash_board(nonce, ships);
    // x equal to BOARD_SIZE is out of bounds
    let x = BOARD_SIZE as u8;
    let y = 0;
    main(nonce, ships, board_hash, x, y, false);
}

#[test(should_fail)]
fn test_out_of_bounds_y_should_fail() {
    let nonce = 1112;
    let ships = [[2, 2, 0], [4, 0, 1], [1, 0, 0], [5, 5, 1], [6, 3, 0]];
    let board_hash = hash_board(nonce, ships);
    // y equal to BOARD_SIZE is out of bounds
    let x = 0;
    let y = BOARD_SIZE as u8;
    main(nonce, ships, board_hash, x, y, false);
}

#[test(should_fail)]
fn test_hash_mismatch_should_fail() {
    let nonce = 12345;
    let ships = [[2, 2, 0], [4, 0, 1], [1, 0, 0], [5, 5, 1], [6, 3, 0]];
    // Intentionally wrong hash
    let wrong_board_hash = 0;
    main(nonce, ships, wrong_board_hash, 0, 0, false);
}

#[test(should_fail_with = "Invalid direction")]
fn test_invalid_direction_in_ships_should_fail() {
    let nonce = 2024;
    // Third ship has invalid direction 2
    let ships = [[2, 2, 0], [4, 0, 1], [1, 0, 2], [5, 5, 1], [6, 3, 0]];
    let board_hash = hash_board(nonce, ships);
    main(nonce, ships, board_hash, 1, 1, false);
}

#[test(should_fail)]
fn test_wrong_hit_flag_should_fail() {
    let nonce = 9999;
    let x0 = BOARD_SIZE as u8 - SHIP_LENGTHS[0];
    let ships = [
        [x0, 0, SHIP_DIRECTION_RIGHT],
        [0, 2, SHIP_DIRECTION_DOWN],
        [3, 3, SHIP_DIRECTION_RIGHT],
        [6, 1, SHIP_DIRECTION_DOWN],
        [1, BOARD_SIZE as u8 - 1, SHIP_DIRECTION_RIGHT],
    ];
    let board_hash = hash_board(nonce, ships);

    // Choose a miss but claim it's a hit
    let x = 0;
    let y = BOARD_SIZE as u8 - 1;
    main(nonce, ships, board_hash, x, y, true);
}
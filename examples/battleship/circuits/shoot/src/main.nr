use common::{
    X_INDEX, Y_INDEX, BOARD_SIZE, DIRECTION_INDEX, NUMBER_OF_SHIPS, SHIP_DIRECTION_DOWN,
    SHIP_DIRECTION_RIGHT, SHIP_LENGTHS, hash_board, 
};

fn main(nonce: Field, ships: [[u8; 3]; NUMBER_OF_SHIPS], board_hash: pub Field, shot_x: pub u8, shot_y: pub u8) {
    // validate the guess is actually valid
    assert(shot_x >= 0 & shot_x < BOARD_SIZE as u8);
    assert(shot_y >= 0 & shot_y < BOARD_SIZE as u8);

    // validate the inputted ships matches the public hash
    assert(board_hash == hash_board(nonce, ships));

    // check if it's a hit to any of the ships
    let hit0 = is_hit(shot_x, shot_y, ships[0], SHIP_LENGTHS[0]);
    let hit1 = is_hit(shot_x, shot_y, ships[1], SHIP_LENGTHS[1]);
    let hit2 = is_hit(shot_x, shot_y, ships[2], SHIP_LENGTHS[2]);
    let hit3 = is_hit(shot_x, shot_y, ships[3], SHIP_LENGTHS[3]);
    let hit4 = is_hit(shot_x, shot_y, ships[4], SHIP_LENGTHS[4]);
    assert(hit0 | hit1 | hit2 | hit3 | hit4);
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
    let shot_x = 1;
    let shot_y = 2;

    main(nonce, ships, board_hash, shot_x, shot_y);
}

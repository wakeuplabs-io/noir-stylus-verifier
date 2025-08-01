use common::{
    BOARD_SIZE, DIRECTION_INDEX, NUMBER_OF_SHIPS, SHIP_DIRECTION_DOWN,
    SHIP_DIRECTION_RIGHT, SHIP_LENGTHS, hash_board, 
};

fn main(nonce: Field, ships: [[u8; 3]; NUMBER_OF_SHIPS], board_hash: pub Field, shot: pub [u8; 2]) {
    // validate the guess is actually valid
    assert(shot[0] >= 0 & shot[0] < BOARD_SIZE as u8);
    assert(shot[1] >= 0 & shot[1] < BOARD_SIZE as u8);

    // validate the inputted ships matches the public hash
    assert(board_hash == hash_board(nonce, ships));

    // check if it's a hit
    let hit0 = is_hit(shot, ships[0], SHIP_LENGTHS[0]);
    let hit1 = is_hit(shot, ships[1], SHIP_LENGTHS[1]);
    let hit2 = is_hit(shot, ships[2], SHIP_LENGTHS[2]);
    let hit3 = is_hit(shot, ships[3], SHIP_LENGTHS[3]);
    let hit4 = is_hit(shot, ships[4], SHIP_LENGTHS[4]);
    assert(hit0 | hit1 | hit2 | hit3 | hit4);
}

fn is_hit(guess: [u8; 2], ship: [u8; 3], len: u8) -> bool {
    let mut is_hit = false;
    if (ship[DIRECTION_INDEX] == SHIP_DIRECTION_DOWN) {
        let x_match = guess[0] == ship[0];
        let y_in_range = (guess[1] >= ship[1]) & (guess[1] < ship[1] + len);
        is_hit = x_match & y_in_range;
    } else if (ship[DIRECTION_INDEX] == SHIP_DIRECTION_RIGHT) {
        let y_match = guess[1] == ship[1];
        let x_in_range = (guess[0] >= ship[0]) & (guess[0] < ship[0] + len);
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
    let shot = [1, 2];

    main(nonce, ships, board_hash, shot);
}

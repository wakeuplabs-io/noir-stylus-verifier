use common::{
    EMPTY_BOARD,
    NUMBER_OF_SHIPS,
    SHIP_LENGTHS,
    SHIP_DIRECTION_DOWN,
    SHIP_DIRECTION_RIGHT,
    X_INDEX,
    Y_INDEX,
    BOARD_SIZE,
    DIRECTION_INDEX,
    hash_board,
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

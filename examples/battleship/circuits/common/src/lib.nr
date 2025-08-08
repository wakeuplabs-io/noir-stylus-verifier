use poseidon::poseidon2::Poseidon2::hash;

pub global BOARD_SIZE: u32 = 10;

pub global NUMBER_OF_SHIPS: u32 = 5;

pub global EMPTY_BOARD: [[Field; BOARD_SIZE]; BOARD_SIZE] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

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
    // ie, [3,2,1] would become "123"
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

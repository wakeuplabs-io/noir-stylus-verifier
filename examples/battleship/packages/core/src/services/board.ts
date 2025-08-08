import { BOARD_SIZE } from "../config/constants";

export type ShipPlacement = {
  x: number;
  y: number;
  direction: Direction;
};

export enum Direction {
  HORIZONTAL = "horizontal",
  VERTICAL = "vertical",
}

export enum ShipType {
  CARRIER = "carrier",
  BATTLESHIP = "battleship",
  CRUISER = "cruiser",
  SUBMARINE = "submarine",
  DESTROYER = "destroyer",
}

export const SHIP_LENGTHS: Record<ShipType, number> = {
  [ShipType.CARRIER]: 5,
  [ShipType.BATTLESHIP]: 4,
  [ShipType.CRUISER]: 3,
  [ShipType.SUBMARINE]: 3,
  [ShipType.DESTROYER]: 2,
};

export type BoardShips = Record<ShipType, ShipPlacement>;

export enum BoardCellState {
  EMPTY = "empty",
  SHIP = "ship",
  HIT = "hit",
  MISS = "miss",
}

export class Board {
  /*
   * Create an empty 10x10 board
   * @returns The empty board.
   */
  static empty = (): BoardCellState[][] => {
    return Array.from({ length: BOARD_SIZE }, () =>
      Array(BOARD_SIZE).fill(BoardCellState.EMPTY)
    );
  };

  /*
   * Build 10x10 board from ships
   * @param ships - The ships on the board.
   * @returns The board.
   */
  static fromShips = (ships: BoardShips): BoardCellState[][] => {
    const board = this.empty();

    for (const shipType of Object.values(ShipType)) {
      const ship = ships[shipType];
      const length = SHIP_LENGTHS[shipType];

      for (let l = 0; l < length; l++) {
        const x_a = ship.direction === Direction.VERTICAL ? 0 : l;
        const y_a = ship.direction === Direction.VERTICAL ? l : 0;
        const x = ship.x + x_a;
        const y = ship.y + y_a;

        if (board[y][x] !== BoardCellState.EMPTY) {
          throw new Error("Ship overlaps with another ship");
        }

        board[y][x] = BoardCellState.SHIP;
      }
    }

    return board;
  };

  static random = (): { ships: BoardShips; board: BoardCellState[][] } => {
    const ships: Partial<BoardShips> = {};
    const board = this.empty();

    // Helper function to check if a ship placement is valid
    const isValidPlacement = (x: number, y: number, length: number, direction: Direction): boolean => {
      // Check bounds
      if (direction === Direction.HORIZONTAL) {
        if (x + length > BOARD_SIZE || y >= BOARD_SIZE) return false;
      } else {
        if (x >= BOARD_SIZE || y + length > BOARD_SIZE) return false;
      }

      // Check for overlaps
      for (let i = 0; i < length; i++) {
        const checkX = direction === Direction.HORIZONTAL ? x + i : x;
        const checkY = direction === Direction.VERTICAL ? y + i : y;
        
        if (board[checkY][checkX] !== BoardCellState.EMPTY) {
          return false;
        }
      }

      return true;
    };

    // Helper function to place a ship on the board
    const placeShip = (x: number, y: number, length: number, direction: Direction): void => {
      for (let i = 0; i < length; i++) {
        const placeX = direction === Direction.HORIZONTAL ? x + i : x;
        const placeY = direction === Direction.VERTICAL ? y + i : y;
        board[placeY][placeX] = BoardCellState.SHIP;
      }
    };

    // Place ships one by one, ensuring no overlaps
    for (const shipType of Object.values(ShipType)) {
      const length = SHIP_LENGTHS[shipType];
      let placed = false;
      let attempts = 0;
      const maxAttempts = 1000; // Prevent infinite loops

      while (!placed && attempts < maxAttempts) {
        const direction = Math.random() < 0.5 ? Direction.HORIZONTAL : Direction.VERTICAL;
        
        // Calculate proper bounds based on direction
        const maxX = direction === Direction.HORIZONTAL ? BOARD_SIZE - length : BOARD_SIZE - 1;
        const maxY = direction === Direction.VERTICAL ? BOARD_SIZE - length : BOARD_SIZE - 1;
        
        const x = Math.floor(Math.random() * (maxX + 1));
        const y = Math.floor(Math.random() * (maxY + 1));

        if (isValidPlacement(x, y, length, direction)) {
          ships[shipType] = { x, y, direction };
          placeShip(x, y, length, direction);
          placed = true;
        }
        
        attempts++;
      }

      if (!placed) {
        throw new Error(`Could not place ship ${shipType} after ${maxAttempts} attempts`);
      }
    }
    
    return { ships: ships as BoardShips, board };
  };
}

export const BOARD_SIZE = 10;

export type Direction = "horizontal" | "vertical";
export type CellState = "empty" | "ship" | "hit" | "miss";

export type ShipPlacement = {
  name: string;
  size: number;
  direction: "horizontal" | "vertical";
  start: { row: number; col: number };
  end: { row: number; col: number };
};

export type PlacementResult = {
  board: CellState[][];
  ships: ShipPlacement[];
};


export type Ship = {
  name: string;
  size: number;
};

const ships: Ship[] = [
  { name: "Carrier", size: 5 },
  { name: "Battleship", size: 4 },
  { name: "Cruiser", size: 3 },
  { name: "Submarine", size: 3 },
  { name: "Destroyer", size: 2 },
];

export const createEmptyBoard = (): CellState[][] =>
  Array.from({ length: BOARD_SIZE }, () => Array(BOARD_SIZE).fill("empty"));


export const placeShipsRandomly = (): PlacementResult => {
  const board = createEmptyBoard();
  const placedShips: ShipPlacement[] = [];

  for (const ship of ships) {
    let placed = false;

    while (!placed) {
      const direction: "horizontal" | "vertical" = Math.random() < 0.5 ? "horizontal" : "vertical";
      const row = Math.floor(Math.random() * BOARD_SIZE);
      const col = Math.floor(Math.random() * BOARD_SIZE);

      const fits =
        direction === "horizontal"
          ? col + ship.size <= BOARD_SIZE
          : row + ship.size <= BOARD_SIZE;

      if (!fits) continue;

      // Check overlap
      let canPlace = true;
      for (let i = 0; i < ship.size; i++) {
        const r = direction === "horizontal" ? row : row + i;
        const c = direction === "horizontal" ? col + i : col;
        if (board[r][c] !== "empty") {
          canPlace = false;
          break;
        }
      }

      if (!canPlace) continue;

      // Place the ship
      for (let i = 0; i < ship.size; i++) {
        const r = direction === "horizontal" ? row : row + i;
        const c = direction === "horizontal" ? col + i : col;
        board[r][c] = "ship";
      }

      placedShips.push({
        name: ship.name,
        size: ship.size,
        direction,
        start: { row, col },
        end: {
          row: direction === "horizontal" ? row : row + ship.size - 1,
          col: direction === "horizontal" ? col + ship.size - 1 : col,
        },
      });

      placed = true;
    }
  }

  board[0][0] = "hit";
  board[0][1] = "miss";

  return { board, ships: placedShips };
};

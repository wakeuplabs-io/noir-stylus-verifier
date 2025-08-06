import { UltraHonkBackend } from "@aztec/bb.js";
import { Noir } from "@noir-lang/noir_js";
import { toHex } from "viem";
import boardBytecode from "../assets/board_bytecode.json";
import shootBytecode from "../assets/shoot_bytecode.json";
import { poseidon2Hash } from "@zkpassport/poseidon2";
import { SHIP_LENGTHS } from "../config/constants";

export enum BoardCellState {
  EMPTY = 0,
  SHIP = 1,
  HIT = 2,
  MISS = 3,
}

export class BoardCircuit {

  /*
  * Generate a proof for the board circuit.
  * @param nonce - The nonce for the board.
  * @param ships - The ships on the board.
  * @param board_hash - The hash of the board.
  * @returns The proof.
  */
  static async generateProof(
    nonce: bigint,
    ships: Array<[number, number, number]>,
    board_hash: bigint
  ) {
    const noir = new Noir(boardBytecode as any);
    const backend = new UltraHonkBackend(boardBytecode.bytecode);

    const { witness } = await noir.execute({
      nonce: toHex(nonce),
      ships: ships.map((p) => p.map((v) => toHex(v))),
      board_hash: toHex(board_hash),
    });

    const { proof } = await backend.generateProof(witness, {
      keccak: true,
    });

    return proof;
  }

  /*
  * Hash the board with the nonce.
  * Poseidon takes in a series of numbers, so we want to serialize each ship position as a number.
  * We know a Battleship position is (0...9), so we encode (x,y,p) array as a 3-digit number
  * ie, [3,2,1] would become "123"
  * @param nonce - The nonce for the board.
  * @param ships - The ships on the board.
  * @returns The hash of the board.
  */
  static hashBoard(
    nonce: bigint,
    ships: Array<[number, number, number]>
  ): bigint {
    return poseidon2Hash([
      nonce,
      ...ships.map((ship) => BigInt(ship[0] * 100 + ship[1] * 10 + ship[2])),
    ]);
  }

  /*
  * Build 10x10 board from ships
  * @param ships - The ships on the board.
  * @returns The board.
  */
  static buildBoard(ships: Array<[number, number, number]>): Array<Array<BoardCellState>> {
    const board = Array.from({ length: 10 }, () =>
      Array.from({ length: 10 }, () => BoardCellState.EMPTY)
    );
    
    for (let i = 0; i < ships.length; i++) {
      const [x, y, direction] = ships[i];
      const length = SHIP_LENGTHS[i];

      if (direction === 0) {
        for (let j = 0; j < length; j++) {
          if (board[x][y + j] !== BoardCellState.EMPTY) {
            throw new Error("Ship overlaps with another ship");
          }
          board[x][y + j] = BoardCellState.SHIP;
        }
      } else if (direction === 1) {
        for (let j = 0; j < length; j++) {
          if (board[x + j][y] !== BoardCellState.EMPTY) {
            throw new Error("Ship overlaps with another ship");
          }
          board[x + j][y] = BoardCellState.SHIP;
        }
      }
    }

    return board;
  }
}

export class ShootCircuit {
  /*
  * Generate a proof for the shoot circuit.
  * @param nonce - The nonce for the shoot.
  * @param ships - The ships on the board.
  * @param board_hash - The hash of the board.
  * @param x - The x coordinate of the shot.
  * @param y - The y coordinate of the shot.
  * @param hit - Whether the shot hit a ship.
  * @returns The proof.
  */
  static async generateProof(
    nonce: bigint,
    ships: Array<[bigint, bigint, bigint]>,
    board_hash: bigint,
    x: bigint,
    y: bigint,
    hit: boolean
  ) {
    const noir = new Noir(shootBytecode as any);
    const backend = new UltraHonkBackend(shootBytecode.bytecode);

    const { witness } = await noir.execute({
      nonce: toHex(nonce),
      ships: ships.map((p) => p.map((v) => toHex(v))),
      board_hash: toHex(board_hash),
      x: toHex(x),
      y: toHex(y),
      hit: hit ? 1 : 0,
    });

    const { proof } = await backend.generateProof(witness, {
      keccak: true,
    });


    return proof;
  }
}

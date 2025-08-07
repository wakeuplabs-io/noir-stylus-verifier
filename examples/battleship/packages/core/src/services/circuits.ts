import { UltraHonkBackend } from "@aztec/bb.js";
import { Noir } from "@noir-lang/noir_js";
import { toHex } from "viem";
import boardBytecode from "../assets/board_bytecode.json";
import shootBytecode from "../assets/shoot_bytecode.json";
import { poseidon2Hash } from "@zkpassport/poseidon2";
import { BoardShips, Direction, ShipType } from "./board";

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
    ships: BoardShips,
    boardHash: bigint
  ) {
    const noir = new Noir(boardBytecode as any);
    const backend = new UltraHonkBackend(boardBytecode.bytecode);

    const { witness } = await noir.execute({
      nonce: toHex(nonce),
      ships: formatShips(ships).map((ship) => ship.map((v) => toHex(v))),
      board_hash: toHex(boardHash),
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
  static hashBoard(nonce: bigint, ships: BoardShips): bigint {
    return poseidon2Hash([
      nonce,
      ...formatShips(ships).map((ship) =>
        BigInt(ship[0] * 100 + ship[1] * 10 + ship[2])
      ),
    ]);
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
    ships: BoardShips,
    boardHash: bigint,
    x: bigint,
    y: bigint,
    hit: boolean
  ) {
    const noir = new Noir(shootBytecode as any);
    const backend = new UltraHonkBackend(shootBytecode.bytecode);

    const { witness } = await noir.execute({
      nonce: toHex(nonce),
      ships: formatShips(ships).map((ship) => ship.map((v) => toHex(v))),
      board_hash: toHex(boardHash),
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

/*
 * Format the ships as expected by the circuit.
 * @param ships - The ships on the board.
 * @returns The formatted ships.
 */
function formatShips(ships: BoardShips): Array<[number, number, number]> {
  return [
    [
      ships[ShipType.CARRIER].x,
      ships[ShipType.CARRIER].y,
      ships[ShipType.CARRIER].direction === Direction.HORIZONTAL ? 1 : 0,
    ],
    [
      ships[ShipType.BATTLESHIP].x,
      ships[ShipType.BATTLESHIP].y,
      ships[ShipType.BATTLESHIP].direction === Direction.HORIZONTAL ? 1 : 0,
    ],
    [
      ships[ShipType.CRUISER].x,
      ships[ShipType.CRUISER].y,
      ships[ShipType.CRUISER].direction === Direction.HORIZONTAL ? 1 : 0,
    ],
    [
      ships[ShipType.SUBMARINE].x,
      ships[ShipType.SUBMARINE].y,
      ships[ShipType.SUBMARINE].direction === Direction.HORIZONTAL ? 1 : 0,
    ],
    [
      ships[ShipType.DESTROYER].x,
      ships[ShipType.DESTROYER].y,
      ships[ShipType.DESTROYER].direction === Direction.HORIZONTAL ? 1 : 0,
    ],
  ];
}

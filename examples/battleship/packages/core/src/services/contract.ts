import {
  Address,
  checksumAddress,
  createPublicClient,
  decodeEventLog,
  encodeEventTopics,
  encodeFunctionData,
  http,
  keccak256,
  numberToBytes,
  PublicClient,
  toHex,
} from "viem";
import { BattleshipContractAbi } from "../config/abi";
import { MULTICALL_ADDRESS, SupportedChainId } from "../config/constants";
import { Move } from "../types/game";

export class BattleshipContract {
  private address: Address;
  private chainId: SupportedChainId;
  private publicClient: PublicClient;

  constructor(
    chainId: SupportedChainId,
    address: `0x${string}`,
    rpcUrl: string
  ) {
    this.address = checksumAddress(address);
    this.chainId = chainId;

    this.publicClient = createPublicClient({
      transport: http(rpcUrl),
    });
  }

  getGameId(join_code: bigint): bigint {
    return BigInt(keccak256(numberToBytes(join_code, { size: 32 })));
  }

  async prepareCreateGame(
    join_code: bigint,
    board_hash: bigint,
    proof: Uint8Array<ArrayBufferLike>,
  ) {
    const txRequest = await this.publicClient.prepareTransactionRequest({
      to: this.address,
      data: encodeFunctionData({
        abi: BattleshipContractAbi,
        functionName: "createGame",
        args: [
          this.getGameId(join_code), 
          board_hash,
          `0x${Array.from(proof, (byte) =>
            byte.toString(16).padStart(2, "0")
          ).join("")}`,
        ],
      }),
      value: 0n,
      chain: null,
    });

    return txRequest;
  }

  /**
   * Recovers the game ID from a transaction hash
   * @param hash - The hash of the transaction
   * @returns The game ID
   */
  async recoverGameId(hash: `0x${string}`): Promise<bigint> {
    const receipt = await this.publicClient.waitForTransactionReceipt({
      hash: hash,
    });
    const [gameCreatedTopic] = encodeEventTopics({
      abi: BattleshipContractAbi,
      eventName: "GameCreated",
    });

    // Find and decode the matching log
    const log = receipt.logs.find((log) => log.topics[0] === gameCreatedTopic);
    if (!log) throw new Error("Log not found");

    const decoded = decodeEventLog({
      abi: BattleshipContractAbi,
      data: log.data,
      topics: log.topics,
    });

    return (decoded.args as any).gameId;
  }

  /**
   * Prepares a transaction to join a game
   * @param join_code - The join code, this hashed to create the game_id
   * @param board_hash - The hash of the board
   * @param proof - The proof of the board
   */
  async prepareJoinGame(
    join_code: bigint,
    board_hash: bigint,
    proof: Uint8Array<ArrayBufferLike>,
  ) {
    const txRequest = await this.publicClient.prepareTransactionRequest({
      to: this.address,
      data: encodeFunctionData({
        abi: BattleshipContractAbi,
        functionName: "joinGame",
        args: [
          `0x${Array.from(proof, (byte) =>
            byte.toString(16).padStart(2, "0")
          ).join("")}`,
          board_hash,
          join_code,
        ],
      }),
      value: 0n,
      chain: null,
    });

    return txRequest;
  }

  async prepareShoot(
    gameId: bigint,
    proof: Uint8Array<ArrayBufferLike>,
    previous_move_hit: boolean,
    previous_move_x: bigint,
    previous_move_y: bigint,
    x: bigint,  
    y: bigint
  ) {
    const txRequest = await this.publicClient.prepareTransactionRequest({
      to: this.address,
      data: encodeFunctionData({
        abi: BattleshipContractAbi,
        functionName: "shoot",
        args: [
          BigInt(gameId),
          `0x${Array.from(proof, (byte) =>
            byte.toString(16).padStart(2, "0")
          ).join("")}`,
          previous_move_hit,
          previous_move_x,
          previous_move_y,
          x,
          y,
        ],
      }),
      value: 0n,
      chain: null,
    });

    return txRequest;
  }

  /// Waits for the user turn to be reached and returns the opponents move index.
  /// it returns opponent move, if it's the first move, it returns -1
  async waitForUserTurn(gameId: bigint, isPlayer1: boolean): Promise<number> {
    let moveCount = await this.getGameMoveCount(gameId);

    // player 1 goes first.
    if (moveCount === 0n && isPlayer1) {
      return -1;
    }

    while (true) {
      if (isPlayer1 && moveCount % 2n === 0n) {
        return Number(moveCount) - 1;
      } else if (!isPlayer1 && moveCount % 2n === 1n) {
        return Number(moveCount) - 1;
      } else {
        await new Promise((resolve) => setTimeout(resolve, 1000));
        moveCount = await this.getGameMoveCount(gameId);
      }
    }
  }

  /// Waits for the move with index moveIndex to be made
  async waitForMove(gameId: bigint, moveIndex: bigint): Promise<Move> {
    let moveCount = await this.getGameMoveCount(gameId);
    while (moveCount < BigInt(moveIndex)) {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      moveCount = await this.getGameMoveCount(gameId);
    }
    return await this.getGameMove(gameId, moveIndex);
  }

  /// Waits for the 2 players to join
  async waitForPlayersToJoin(gameId: bigint): Promise<{
    player1: Address;
    player2: Address;
  }> {
    let { player1, player2 } = await this.getGamePlayers(gameId);
    while (player2 === "0x0000000000000000000000000000000000000000") {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      ({ player1, player2 } = await this.getGamePlayers(gameId));
    }

    return { player1, player2 };
  }

  async getGamePlayers(gameId: bigint): Promise<{
    player1: Address;
    player2: Address;
  }> {
    const res = await this.publicClient.readContract({
      address: this.address,
      abi: BattleshipContractAbi,
      functionName: "getGamePlayers",
      args: [gameId],
    });

    return {
      player1: res[0],
      player2: res[1],
    };
  }

  async getGameBoardsHashes(gameId: bigint): Promise<{
    player1BoardHash: bigint;
    player2BoardHash: bigint;
  }> {
    const res = await this.publicClient.readContract({
      address: this.address,
      abi: BattleshipContractAbi,
      functionName: "getGameBoardsHashes",
      args: [gameId],
    });

    return {
      player1BoardHash: res[0],
      player2BoardHash: res[1],
    };
  }

  async getGameMoveCount(gameId: bigint): Promise<bigint> {
    return this.publicClient.readContract({
      address: this.address,
      abi: BattleshipContractAbi,
      functionName: "getGameMoveCount",
      args: [gameId],
    });
  }

  async getGameMove(gameId: bigint, moveIndex: bigint): Promise<Move> {
    const res = await this.publicClient.readContract({
      address: this.address,
      abi: BattleshipContractAbi,
      functionName: "getGameMove",
      args: [gameId, moveIndex],
    });

    return {
      x: res[0],
      y: res[1],
      isHit: res[2],
    };
  }
}

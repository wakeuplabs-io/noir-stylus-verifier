#!/usr/bin/env tsx

import "dotenv/config";
import { Command } from "commander";
import {
  BattleshipContract,
  BoardCellState,
  BoardCircuit,
  ShootCircuit,
} from "@battleship/core";
import {
  createWalletClient,
  decodeErrorResult,
  http,
  toHex,
} from "viem";
import { privateKeyToAccount, privateKeyToAddress } from "viem/accounts";
import * as prompts from "@clack/prompts";
import { RPC_URL, CONTRACT_ADDRESS, CHAIN_ID } from "./env";
import fs from "fs";
import { BattleshipContractAbi } from "@battleship/core/src/config/abi";

const contract = new BattleshipContract(CHAIN_ID, CONTRACT_ADDRESS, RPC_URL);

export const walletClient = createWalletClient({
  transport: http(RPC_URL),
});

const program = new Command();

// Basic interface for the battleship game, creates battleship.json file by default with

program
  .name("cli")
  .description("Zero Knowledge Battleship with Noir and Stylus")
  .version("0.1.0");

program
  .command("create")
  .requiredOption("-j, --join-code <joinCode>", "Join code for the game")
  .requiredOption("--private-key <privateKey>", "Private key for the player")
  .option("-b, --board <path>", "Path to the board file", "board.json")
  .description("Create a new game")
  .action(async (options) => {
    const board = JSON.parse(fs.readFileSync(options.board, "utf8"));
    const nonce = BigInt(board.nonce);
    const ships = board.ships.map((ship: [string, string, string]) =>
      ship.map(Number)
    );

    const boardHash = BoardCircuit.hashBoard(nonce, ships);

    const proof = await BoardCircuit.generateProof(nonce, ships, boardHash);

    const tx = await contract.prepareCreateGame(
      proof,
      boardHash,
      BigInt(options.joinCode)
    );

    console.log(`Creating game with board hash: ${toHex(boardHash)}`);

    const txHash = await walletClient.sendTransaction({
      ...tx,
      account: privateKeyToAccount(options.privateKey),
    });

    console.log(`Game created in tx ${txHash}`);

    const gameId = await contract.recoverGameId(txHash);

    console.log(`Game created with ID: ${gameId}`);
  });

program
  .command("join")
  .argument("<gameId>", "Game ID")
  .requiredOption("-j, --join-code <joinCode>", "Join code for the game")
  .requiredOption("--private-key <privateKey>", "Private key for the player")
  .option("-b, --board <path>", "Path to the board file", "board.json")
  .description("Join a game")
  .action(async (gameId, options) => {
    gameId = BigInt(gameId);
    const joinCode = BigInt(options.joinCode);
    const board = JSON.parse(fs.readFileSync(options.board, "utf8"));
    const nonce = BigInt(board.nonce);
    const ships = board.ships.map((ship: [string, string, string]) =>
      ship.map(Number)
    );

    const boardHash = BoardCircuit.hashBoard(nonce, ships);

    console.log(`Board hash: ${toHex(boardHash)}`);

    const proof = await BoardCircuit.generateProof(nonce, ships, boardHash);

    const tx = await contract.prepareJoinGame(
      gameId,
      proof,
      boardHash,
      joinCode
    );

    console.log(`Joining game with ID: ${gameId}`);

    try {
      const txHash = await walletClient.sendTransaction({
        ...tx,
        account: privateKeyToAccount(options.privateKey),
      });
      console.log(`Game joined in tx ${txHash}`);
    } catch (err: any) {
      if (err?.data) {
        try {
          const decoded = decodeErrorResult({
            abi: BattleshipContractAbi, // your contract's ABI
            data: err.data,
          });
          console.error("Decoded error:", decoded);
        } catch (decodeErr) {
          console.error("Could not decode error:", err);
        }
      } else {
        console.error("Unknown error:", err);
      }
    }
  });

program
  .command("get")
  .argument("<gameId>", "Game ID")
  .description("Get game state")
  .action(async (gameId) => {
    gameId = BigInt(gameId);
    const { player1, player2 } = await contract.getGamePlayers(gameId);
    console.log(`Player 1: ${player1}`);
    console.log(`Player 2: ${player2}`);
    const { player1BoardHash, player2BoardHash } =
      await contract.getGameBoardsHashes(gameId);
    console.log(`Player 1 board hash: ${toHex(player1BoardHash)}`);
    console.log(`Player 2 board hash: ${toHex(player2BoardHash)}`);
    const moveCount = await contract.getGameMoveCount(gameId);
    console.log(`Move count: ${moveCount}`);
  });

program
  .command("play")
  .argument("<gameId>", "Game ID")
  .requiredOption("--private-key <privateKey>", "Private key for the player")
  .option("-b, --board <path>", "Path to the original board file", "board.json")
  .description("Attack and defend the game")
  .action(async (gameId, options) => {
    gameId = BigInt(gameId);

    // recover our private board
    const boardJson = JSON.parse(fs.readFileSync(options.board, "utf8"));
    const nonce = BigInt(boardJson.nonce);
    const ships = boardJson.ships.map((ship: [string, string, string]) =>
      ship.map(Number)
    );
    const boardHash = await BoardCircuit.hashBoard(nonce, ships);

    // Recover game state, our board and shots
    let board: BoardCellState[][];
    let opponentBoard: BoardCellState[][];
    if (fs.existsSync(`${gameId}-game-board.json`)) {
      const gameState = JSON.parse(
        fs.readFileSync(`${gameId}-game-board.json`, "utf8")
      );
      board = gameState.ours;
      opponentBoard = gameState.opponent;
    } else {
      board = BoardCircuit.buildBoard(ships);
      opponentBoard = Array.from({ length: 10 }, () =>
        Array.from({ length: 10 }, () => BoardCellState.EMPTY)
      );
    }

    console.log(`Waiting for players to join...`);

    const player = privateKeyToAddress(options.privateKey);
    const { player1, player2 } = await contract.getGamePlayers(gameId);

    let isPlayer1 = false;
    if (player1 === player) {
      isPlayer1 = true;
    } else if (player2 === player) {
      isPlayer1 = false;
    } else {
      throw new Error("You are not a player in this game");
    }

    console.log(`Game ID: ${gameId}`);
    console.log(`Player 1${isPlayer1 ? " (you)" : ""}: ${player1}`);
    console.log(`Player 2${!isPlayer1 ? " (you)" : ""}: ${player2}`);

    while (true) {
      console.log(`Waiting for your turn...`);
      const opponentMoveIndex = await contract.waitForUserTurn(
        gameId,
        isPlayer1
      );

      let isPreviousMoveHit;
      let isPreviousMoveHitProof: Uint8Array<ArrayBufferLike>;
      let opponentMove: { x: bigint; y: bigint };
      if (opponentMoveIndex < 0) {
        // first move
        isPreviousMoveHit = false;
        isPreviousMoveHitProof = new Uint8Array(0);
        opponentMove = { x: 0n, y: 0n };
      } else {
        // check our previous move
        if (opponentMoveIndex !== 0) {
          const ourMove = await contract.getGameMove(
            gameId,
            BigInt(opponentMoveIndex - 1)
          );
          if (ourMove.isHit) {
            console.log("Our previous move was a hit");
            opponentBoard[Number(ourMove.x)][Number(ourMove.y)] =
              BoardCellState.HIT;
          } else {
            console.log("Our previous move was a miss");
            opponentBoard[Number(ourMove.x)][Number(ourMove.y)] =
              BoardCellState.MISS;
          }
        }

        // get opponent move from contract and certify
        opponentMove = await contract.getGameMove(
          gameId,
          BigInt(opponentMoveIndex as number)
        );
        const isHit =
          board[Number(opponentMove.x)][Number(opponentMove.y)] ===
          BoardCellState.SHIP;

        console.log(`Opponent move: ${opponentMove.x}, ${opponentMove.y}`);
        if (isHit) {
          console.log("This is a hit");
          board[Number(opponentMove.x)][Number(opponentMove.y)] =
            BoardCellState.HIT;
          isPreviousMoveHit = true;
        } else {
          console.log("This is a miss");
          board[Number(opponentMove.x)][Number(opponentMove.y)] =
            BoardCellState.MISS;
          isPreviousMoveHit = false;
        }

        isPreviousMoveHitProof = await ShootCircuit.generateProof(
          nonce,
          ships,
          boardHash,
          opponentMove.x,
          opponentMove.y,
          isPreviousMoveHit
        );
      }

      // TODO: check if game is over

      let shot: [bigint, bigint];
      while (true) {
        const input = (
          (await prompts.text({
            message: "Enter your shot [A-J][0-9]:",
          })) as string
        ).split("");
        const x = BigInt("ABCDEFGHIJ".indexOf(input[0]));
        const y = BigInt(input[1]);

        if (x < 0n || x > 9n || y < 0n || y > 9n) {
          console.log("Shot out of bounds");
          continue;
        } else if (
          opponentBoard[Number(x)][Number(y)] !== BoardCellState.EMPTY
        ) {
          console.log("You already shot here");
          continue;
        } else {
          shot = [x, y];
          break;
        }
      }

      const tx = await contract.prepareShoot(
        gameId,
        isPreviousMoveHitProof,
        isPreviousMoveHit,
        opponentMove.x,
        opponentMove.y,
        shot[0],
        shot[1]
      );

      const txHash = await walletClient.sendTransaction({
        ...tx,
        account: privateKeyToAccount(options.privateKey),
      });

      console.log(`Shot sent in tx ${txHash}`);

      // store game state
      fs.writeFileSync(
        `${gameId}-game-board.json`,
        JSON.stringify(
          {
            ours: board,
            opponent: opponentBoard,
          },
          null,
          2
        )
      );
    }
  });

program.parse(process.argv);

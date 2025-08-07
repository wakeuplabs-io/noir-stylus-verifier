#!/usr/bin/env tsx

import "dotenv/config";
import { Command } from "commander";
import {
  BattleshipContract,
  Board,
  BoardCellState,
  BoardCircuit,
  ShootCircuit,
} from "@battleship/core";
import { createPublicClient, createWalletClient, http, toHex } from "viem";
import { privateKeyToAccount, privateKeyToAddress } from "viem/accounts";
import * as prompts from "@clack/prompts";
import { RPC_URL, CONTRACT_ADDRESS, CHAIN_ID } from "./env";
import fs from "fs";
import color from "picocolors";
import { arbitrumSepolia } from "viem/chains";

const contract = new BattleshipContract(CHAIN_ID, CONTRACT_ADDRESS, RPC_URL);

export const walletClient = createWalletClient({
  transport: http(RPC_URL),
});
export const publicClient = createPublicClient({
  chain: arbitrumSepolia,
  transport: http(RPC_URL),
});

const program = new Command();

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
    prompts.intro(color.inverse("ZK Battleship by Wakeup Labs ◌○●"));
    const spinner = prompts.spinner();

    const board = JSON.parse(fs.readFileSync(options.board, "utf8"));
    const boardHash = BoardCircuit.hashBoard(BigInt(board.nonce), board.ships);

    spinner.start("Generating board proof...");

    const proof = await BoardCircuit.generateProof(
      BigInt(board.nonce),
      board.ships,
      boardHash
    );

    spinner.stop("Board proof generated");

    spinner.start("Creating game...");

    const tx = await contract.prepareCreateGame(
      BigInt(options.joinCode),
      boardHash,
      proof
    );

    const txHash = await walletClient.sendTransaction({
      ...tx,
      account: privateKeyToAccount(options.privateKey),
    });
    const gameId = contract.getGameId(BigInt(options.joinCode));

    spinner.stop(`Game created with ID: ${toHex(gameId)} in tx: ${txHash}`);

    prompts.outro(`You're all set!`);
  });

program
  .command("join")
  .requiredOption("-j, --join-code <joinCode>", "Join code for the game")
  .requiredOption("--private-key <privateKey>", "Private key for the player")
  .option("-b, --board <path>", "Path to the board file", "board.json")
  .description("Join a game")
  .action(async (options) => {
    prompts.intro(color.inverse("ZK Battleship by Wakeup Labs ◌○●"));
    const spinner = prompts.spinner();

    const joinCode = BigInt(options.joinCode);
    const board = JSON.parse(fs.readFileSync(options.board, "utf8"));
    const boardHash = BoardCircuit.hashBoard(BigInt(board.nonce), board.ships);

    spinner.start("Generating board proof...");

    const proof = await BoardCircuit.generateProof(
      BigInt(board.nonce),
      board.ships,
      boardHash
    );

    spinner.stop("Board proof generated");

    spinner.start(`Joining game...`);

    const tx = await contract.prepareJoinGame(joinCode, boardHash, proof);

    const txHash = await walletClient.sendTransaction({
      ...tx,
      account: privateKeyToAccount(options.privateKey),
    });

    const gameId = contract.getGameId(joinCode);

    spinner.stop(`Game joined in tx ${txHash} with ID: ${toHex(gameId)}`);

    prompts.outro(`You're all set!`);
  });

program
  .command("get")
  .argument("<gameId>", "Game ID")
  .description("Get game state")
  .action(async (gameId) => {
    prompts.intro(color.inverse("ZK Battleship by Wakeup Labs ◌○●"));
    const spinner = prompts.spinner();

    spinner.start("Getting game state...");

    gameId = BigInt(gameId);
    const { player1, player2 } = await contract.getGamePlayers(gameId);
    const { player1BoardHash, player2BoardHash } =
      await contract.getGameBoardsHashes(gameId);
    const moveCount = await contract.getGameMoveCount(gameId);

    spinner.stop(`Game state retrieved`);

    prompts.outro(
      `Retrieved game state:\n\tPlayer 1: ${player1}\n\tPlayer 2: ${player2}\n\tPlayer 1 board hash: ${toHex(
        player1BoardHash
      )}\n\tPlayer 2 board hash: ${toHex(
        player2BoardHash
      )}\n\tMove count: ${moveCount}`
    );
  });

program
  .command("play")
  .argument("<gameId>", "Game ID")
  .requiredOption("--private-key <privateKey>", "Private key for the player")
  .option("-b, --board <path>", "Path to the original board file", "board.json")
  .description("Attack and defend the game")
  .action(async (gameId, options) => {
    prompts.intro(color.inverse("ZK Battleship by Wakeup Labs ◌○●"));
    const spinner = prompts.spinner();

    gameId = BigInt(gameId);

    // recover our private board
    const boardJson = JSON.parse(fs.readFileSync(options.board, "utf8"));
    const boardHash = await BoardCircuit.hashBoard(
      BigInt(boardJson.nonce),
      boardJson.ships
    );

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
      board = Board.fromShips(boardJson.ships);
      opponentBoard = Board.empty();
    }

    for (const row of board) {
      console.log(row.map((cell) => cell === BoardCellState.SHIP ? "S" : "E").join(" "));
    }

    spinner.start("Waiting for players to join...");

    const player = privateKeyToAddress(options.privateKey);
    const { player1, player2 } = await contract.waitForPlayersToJoin(gameId);

    spinner.stop("Players joined");

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

    let opponentMoveIndex = -1;
    while (true) {
      console.log(`Waiting for your turn...`);

      // confirm our turn
      opponentMoveIndex = await contract.waitForUserTurn(gameId, isPlayer1);

      let isPreviousMoveHit;
      let isPreviousMoveHitProof: Uint8Array<ArrayBufferLike>;
      let opponentMove: { x: bigint; y: bigint };
      if (opponentMoveIndex < 0) {
        // first move
        isPreviousMoveHit = false;
        isPreviousMoveHitProof = new Uint8Array(0);
        opponentMove = { x: 0n, y: 0n };
      } else {
        // get opponent move from contract and certify
        opponentMove = await contract.getGameMove(
          gameId,
          BigInt(opponentMoveIndex as number)
        );
        const isHit =
          board[Number(opponentMove.y)][Number(opponentMove.x)] ===
          BoardCellState.SHIP;

        console.log(`Opponent move: ${opponentMove.x}, ${opponentMove.y}`);
        if (isHit) {
          console.log("This is a hit");
          board[Number(opponentMove.y)][Number(opponentMove.x)] =
            BoardCellState.HIT;
          isPreviousMoveHit = true;
        } else {
          console.log("This is a miss");
          board[Number(opponentMove.y)][Number(opponentMove.x)] =
            BoardCellState.MISS;
          isPreviousMoveHit = false;
        }

        isPreviousMoveHitProof = await ShootCircuit.generateProof(
          BigInt(boardJson.nonce),
          boardJson.ships,
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
      await publicClient.waitForTransactionReceipt({
        hash: txHash,
      });

      console.log(`Shot sent in tx ${txHash}`);

      console.log("Waiting for opponent's response...");

      // we're actually just waiting for a confirmation from the opponent
      opponentMoveIndex = await contract.waitForUserTurn(gameId, isPlayer1);

      // check our previous move
      if (opponentMoveIndex > 0) {
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

try {
  program.parse(process.argv);
} catch (error) {
  console.error(error);
}

import { createFileRoute, Navigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { BattleshipBoard, BattleshipBoardCell } from "@/components/board";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import {
  ArrowLeftIcon,
  ArrowRightIcon,
  AudioWaveform,
  CheckIcon,
  Shuffle,
  WalletIcon,
} from "lucide-react";
import {
  DialogDescription,
  DialogOverlay,
  DialogTitle,
} from "@radix-ui/react-dialog";
import {
  useAccount,
  usePublicClient,
  useSendTransaction,
  useWatchContractEvent,
  useDisconnect,
} from "wagmi";
import { waitForTransactionReceipt } from "viem/actions";
import { Button } from "@/components/ui/button";
import { shortenAddress } from "@/lib/utils";
import {
  BattleshipContract,
  Board,
  BoardCellState,
  BoardCircuit,
  ShootCircuit,
  SupportedChainId,
  type BoardShips,
} from "@battleship/core";
import { CONTRACT_ADDRESS, RPC_URL } from "@/env";
import { BattleshipContractAbi } from "@battleship/core/src/config/abi";
import { toast } from "sonner";

enum CreateBoardStep {
  JOIN = "join",
  PROOF = "proof",
  SIGN_AND_SEND = "signAndSend",
  WAITING_JOIN = "waiting-join",
  SUCCESS = "success",
}

enum AttackStep {
  WAITING_FOR_TURN = "waiting-for-turn",
  PROOF = "proof",
  SIGN_AND_SEND = "signAndSend",
  WAITING_FOR_CONFIRMATION = "waiting-for-confirmation",
  SUCCESS = "success",
}

type LogType = "create" | "join" | "attack" | "lost" | "win";

type Log = {
  type: LogType;
  from: string;
  txHash: string;
  args?: any;
};

const contract = new BattleshipContract(
  SupportedChainId.ARBITRUM_SEPOLIA,
  CONTRACT_ADDRESS,
  RPC_URL
);

export const Route = createFileRoute("/board")({
  component: RouteComponent,
});

function RouteComponent() {
  const { address, connector } = useAccount();
  const { sendTransactionAsync } = useSendTransaction();
  const wagmiClient = usePublicClient();
  const { disconnect } = useDisconnect();

  // game id for current game
  const [boardId, setBoardId] = useState<bigint | null>(null);

  // attack flow
  const [joinCode, setJoinCode] = useState<string>("");
  const [isAttacking, setIsAttacking] = useState<boolean>(false);
  const [attackStep, setAttackStep] = useState(AttackStep.PROOF);

  // create board flow
  const [isCreatingBoard, setIsCreatingBoard] = useState<boolean>(false);
  const [createBoardStep, setCreateBoardStep] = useState(CreateBoardStep.PROOF);

  // current board to display on screen
  const [currentBoard, setCurrentBoard] = useState<"user" | "opponent">("user");

  // user ships for proof generation, and boards for display and track of hits and misses
  const [isPlayer1, setIsPlayer1] = useState<boolean>(false);
  const [userShips, setUserShips] = useState<BoardShips>();
  const [userBoard, setUserBoard] = useState<BoardCellState[][]>([[]]);
  const [opponentBoard, setOpponentBoard] = useState<BoardCellState[][]>([[]]);
  const [nonce, setNonce] = useState<bigint>(0n);
  const [boardHash, setBoardHash] = useState<bigint>(0n);

  // logs for display of game events
  const [logs, setLogs] = useState<Log[]>([]);

  const onJoinOrCreate = () => {
    setIsCreatingBoard(true);
    setCreateBoardStep(CreateBoardStep.JOIN);
  };

  const shuffleBoard = () => {
    const { ships, board } = Board.random();
    const nonce = BigInt(Math.floor(Math.random() * 1000000));
    const boardHash = BoardCircuit.hashBoard(nonce, ships);

    setNonce(nonce);
    setBoardHash(boardHash);
    setUserShips(ships);
    setUserBoard(board);
    setOpponentBoard(Board.empty());
  };

  const onCreateBoard = async (joinCode: bigint) => {
    try {
      setIsCreatingBoard(true);
      setCreateBoardStep(CreateBoardStep.PROOF);

      const proof = await BoardCircuit.generateProof(
        nonce,
        userShips!,
        boardHash
      );

      setCreateBoardStep(CreateBoardStep.SIGN_AND_SEND);

      const tx = await contract.prepareCreateGame(joinCode, boardHash, proof);

      const txHash = await sendTransactionAsync({ ...tx });
      await waitForTransactionReceipt(wagmiClient!, { hash: txHash });

      const gameId = contract.getGameId(joinCode);
      setBoardId(gameId);

      setCreateBoardStep(CreateBoardStep.WAITING_JOIN);

      await contract.waitForPlayersToJoin(gameId);

      setCreateBoardStep(CreateBoardStep.SUCCESS);

      setCurrentBoard("opponent");
      setIsCreatingBoard(false);
      setIsPlayer1(true);
    } catch (error) {
      console.error(error);
      setCreateBoardStep(CreateBoardStep.JOIN);
      setIsCreatingBoard(false);
      toast.error("Error creating game", {
        description: error instanceof Error ? error.message : "Unknown error",
      });
    }
  };

  const onJoinBoard = async (joinCode: bigint) => {
    try {
      setIsCreatingBoard(true);
      setCreateBoardStep(CreateBoardStep.PROOF);

      const boardHash = BoardCircuit.hashBoard(nonce, userShips!);
      const proof = await BoardCircuit.generateProof(
        nonce,
        userShips!,
        boardHash
      );

      setCreateBoardStep(CreateBoardStep.SIGN_AND_SEND);

      const tx = await contract.prepareJoinGame(joinCode, boardHash, proof);
      const txHash = await sendTransactionAsync({ ...tx });
      await waitForTransactionReceipt(wagmiClient!, { hash: txHash });

      const gameId = contract.getGameId(joinCode);
      setBoardId(gameId);
      await contract.waitForPlayersToJoin(gameId); // just ensuring, we should not need this

      setCreateBoardStep(CreateBoardStep.SUCCESS);

      setCurrentBoard("opponent");
      setIsCreatingBoard(false);
      setIsPlayer1(false);
    } catch (error) {
      console.error(error);
      setCreateBoardStep(CreateBoardStep.JOIN);
      setIsCreatingBoard(false);
      toast.error("Error joining game", {
        description: error instanceof Error ? error.message : "Unknown error",
      });
    }
  };

  const onAttack = async (y: bigint, x: bigint) => {
    try {
      if (x < 0n || x > 9n || y < 0n || y > 9n) {
        throw new Error("Shot out of bounds");
      } else if (opponentBoard[Number(x)][Number(y)] !== BoardCellState.EMPTY) {
        throw new Error("You already shot here");
      }

      setIsAttacking(true);
      setAttackStep(AttackStep.WAITING_FOR_TURN);

      let opponentMoveIndex = await contract.waitForUserTurn(
        boardId!,
        isPlayer1
      );

      setAttackStep(AttackStep.PROOF);

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
          boardId!,
          BigInt(opponentMoveIndex as number)
        );
        const isHit = [BoardCellState.HIT, BoardCellState.MISS].includes(
          userBoard![Number(opponentMove.y)][Number(opponentMove.x)]
        );

        console.log(`Opponent move: ${opponentMove.x}, ${opponentMove.y}`);
        if (isHit) {
          console.log("This is a hit");
          userBoard![Number(opponentMove.y)][Number(opponentMove.x)] =
            BoardCellState.HIT;
          isPreviousMoveHit = true;
        } else {
          console.log("This is a miss");
          userBoard![Number(opponentMove.y)][Number(opponentMove.x)] =
            BoardCellState.MISS;
          isPreviousMoveHit = false;
        }

        isPreviousMoveHitProof = await ShootCircuit.generateProof(
          nonce,
          userShips!,
          boardHash,
          opponentMove.x,
          opponentMove.y,
          isPreviousMoveHit
        );
      }

      setAttackStep(AttackStep.SIGN_AND_SEND);

      const txHash = await sendTransactionAsync(
        contract.prepareShoot(
          boardId!,
          isPreviousMoveHitProof,
          isPreviousMoveHit,
          opponentMove.x,
          opponentMove.y,
          x,
          y
        )
      );
      await waitForTransactionReceipt(wagmiClient!, { hash: txHash });

      setAttackStep(AttackStep.WAITING_FOR_CONFIRMATION);

      // we're actually just waiting for a confirmation from the opponent
      opponentMoveIndex = await contract.waitForUserTurn(boardId!, isPlayer1);

      // check our previous move.
      if (opponentMoveIndex > 0) {
        const ourMove = await contract.getGameMove(
          boardId!,
          BigInt(opponentMoveIndex - 1)
        );
        if (ourMove.isHit) {
          console.log("Our previous move was a hit");
          opponentBoard[Number(ourMove.y)][Number(ourMove.x)] =
            BoardCellState.HIT;
        } else {
          console.log("Our previous move was a miss");
          opponentBoard[Number(ourMove.y)][Number(ourMove.x)] =
            BoardCellState.MISS;
        }

        setLogs((logs) => [
          {
            type: "attack",
            from: address!,
            txHash: txHash,
            args: {
              x,
              y,
              hit: ourMove.isHit
            },
          },
          ...logs,
        ]);
      }

      setAttackStep(AttackStep.SUCCESS);
      setIsAttacking(false);
    } catch (error) {
      console.error(error);
      setAttackStep(AttackStep.WAITING_FOR_TURN);
      setIsAttacking(false);
      toast.error("Error attacking", {
        description: error instanceof Error ? error.message : "Unknown error",
      });
    }
  };

  const onOpenChangeCreateBoard = (open: boolean) => {
    if (!open) {
      setIsCreatingBoard(false);
      setCreateBoardStep(CreateBoardStep.JOIN);
    }
  };

  const onOpenChangeAttack = (open: boolean) => {
    if (!open) {
      setIsAttacking(false);
      setAttackStep(AttackStep.PROOF);
    }
  };

  useWatchContractEvent({
    address: CONTRACT_ADDRESS,
    abi: BattleshipContractAbi,
    eventName: "GameCreated",
    pollingInterval: 1000,
    onLogs: (events) => {
      for (const event of events) {
        if (BigInt(event.args.gameId ?? 0) !== boardId) {
          continue;
        }

        setLogs((logs) => [
          {
            type: "create",
            from: event.args.player as `0x${string}`,
            txHash: event.transactionHash,
            args: {
              player: event.args.player as `0x${string}`,
            },
          },
          ...logs,
        ]);
      }
    },
  });

  useWatchContractEvent({
    address: CONTRACT_ADDRESS,
    abi: BattleshipContractAbi,
    eventName: "GameJoined",
    pollingInterval: 1000,
    onLogs: (events) => {
      for (const event of events) {
        if (BigInt(event.args.gameId ?? 0) !== boardId) {
          continue;
        }

        setLogs((logs) => [
          {
            type: "join",
            from: event.args.player as `0x${string}`,
            txHash: event.transactionHash,
            args: {
              player: event.args.player as `0x${string}`,
            },
          },
          ...logs,
        ]);
      }
    },
  });

  useWatchContractEvent({
    address: CONTRACT_ADDRESS,
    abi: BattleshipContractAbi,
    eventName: "MoveMade",
    pollingInterval: 1000,
    onLogs: (events) => {
      for (const event of events) {
        if (BigInt(event.args.gameId ?? 0) !== boardId) {
          continue;
        }

        if (event.args.player === address) {
          continue;
        }

        const isHit =
          userBoard![Number(event.args.y)][Number(event.args.x)] ===
          BoardCellState.SHIP;

        userBoard![Number(event.args.y)][Number(event.args.x)] = isHit
          ? BoardCellState.HIT
          : BoardCellState.MISS;

        setLogs((logs) => [
          {
            type: "attack",
            from: event.args.player as `0x${string}`,
            txHash: event.transactionHash,
            args: {
              x: event.args.x,
              y: event.args.y,
              hit: isHit,
            },
          },
          ...logs,
        ]);
      }
    },
  });

  useEffect(() => {
    if (!address) {
      return;
    }

    shuffleBoard();
  }, []);

  if (!address) {
    return <Navigate to="/" />;
  }

  if (!userShips || !userBoard || !opponentBoard) {
    return <div>Loading...</div>;
  }

  return (
    <>
      <div className="min-h-screen w-screen py-16">
        <div className="max-w-4xl mx-auto ">
          {currentBoard === "user" ? (
            <div className="mb-14  w-full flex justify-between items-center">
              <h1 className="text-5xl font-extrabold">Your Board</h1>

              {boardId ? (
                <button
                  onClick={() => setCurrentBoard("opponent")}
                  className="relative px-4 py-2 text-lg font-bold text-black bg-player-2 border-2 border-black rounded-full shadow-[2px_2px_0px_#000] active:shadow-none active:translate-x-[2px] active:translate-y-[2px] transition-transform duration-100"
                >
                  Your Opponent's Board <span className="ml-2">→</span>
                </button>
              ) : (
                <button
                  onClick={onJoinOrCreate}
                  className="relative px-4 py-2 text-lg font-bold text-black bg-player-2 border-2 border-black rounded-full shadow-[2px_2px_0px_#000] active:shadow-none active:translate-x-[2px] active:translate-y-[2px] transition-transform duration-100"
                >
                  Create Or Join Game <span className="ml-2">→</span>
                </button>
              )}
            </div>
          ) : (
            <div className="mb-14 w-full flex justify-between items-center">
              <button
                onClick={() => setCurrentBoard("user")}
                className="relative px-4 py-2 text-lg font-bold text-black bg-player-1 border-2 border-black rounded-full shadow-[2px_2px_0px_#000] active:shadow-none active:translate-x-[2px] active:translate-y-[2px] transition-transform duration-100"
              >
                <span className="mr-2">←</span> Your Board
              </button>

              <h1 className="text-5xl font-extrabold">Your Opponent's Board</h1>
            </div>
          )}

          <div className="flex justify-center border-2 border-black rounded-2xl shadow-[2px_2px_0px_#000] overflow-hidden mb-12">
            {currentBoard === "user" ? (
              <BattleshipBoard
                board={userBoard}
                ships={userShips}
                className="border-none shadow-none rounded-r-none rounded-none bg-player-1"
              />
            ) : (
              <BattleshipBoard
                board={opponentBoard}
                onCellClick={onAttack}
                className="border-none shadow-none rounded-r-none rounded-none bg-player-2"
              />
            )}

            <div className="flex-1 border-l-2 border-black p-8 relative">
              <div className="font-bold text-xl">Game Logs...</div>

              <div className="mt-4 space-y-2 overflow-y-auto pb-10 max-h-[300px]">
                {logs.map((log) => (
                  <GameLog key={log.txHash} log={log} />
                ))}

                <div className="text-sm ">
                  Welcome to the game!{" "}
                  <button
                    className="text-blue-500 underline"
                    onClick={shuffleBoard}
                  >
                    refresh
                  </button>{" "}
                  the board until you like what you see. Then create or join a
                  game.
                </div>
              </div>

              <div className="absolute bottom-0 left-0 right-0 p-8 bg-white border-t-2 border-black shadow-[0px_-8px_0px_rgba(0,0,0,0.08)]">
                <div className="font-bold flex items-center gap-1">
                  <ArrowLeftIcon className="w-4 h-4" /> <span>Tip</span>
                </div>
                <div className="mt-2 text-sm">
                  Click on a cell to attack! We'll denote empty, miss and hit as
                  follows:
                </div>
                <div className="flex items-center gap-2 mt-4">
                  <BattleshipBoardCell state={BoardCellState.EMPTY} />
                  <ArrowRightIcon className="w-4 h-4 text-muted-foreground" />
                  <BattleshipBoardCell state={BoardCellState.MISS} />
                  <ArrowRightIcon className="w-4 h-4 text-muted-foreground" />
                  <BattleshipBoardCell state={BoardCellState.HIT} />
                </div>
              </div>
            </div>
          </div>

          <div className="max-w-6xl mx-auto border-2 border-black rounded-2xl shadow-[2px_2px_0px_#000] p-8 pt-14 relative">
            <div className="px-4 flex items-center h-[44px] font-bold absolute border-2 border-black rounded-full shadow-[2px_2px_0px_#000] bg-white top-0 -translate-y-1/2 left-8">
              <span>About</span>
            </div>

            <div className="font-medium">
              This battleship game is a demo project for the{" "}
              <a
                href="https://github.com/wakeuplabs-io/noir-stylus-verifier"
                target="_blank"
                className="text-blue-500 underline"
              >
                Noir Stylus Verifier
              </a>{" "}
              toolkit that allows{" "}
              <a
                href="https://noir-lang.org/"
                target="_blank"
                className="text-blue-500 underline"
              >
                {" "}
                Noir
              </a>{" "}
              verifiers to live in{" "}
              <a
                href="https://arbitrum.io/stylus"
                target="_blank"
                className="text-blue-500 underline"
              >
                Arbitrum Stylus
              </a>
              . To start simply refresh the board until you like the position of
              the ships. Then click on the "Create Or Join Game" button to
              create or join a game. If you're creating a game generate a random
              code and share it with your opponent to start the game. After the
              game started do not leave or refresh the page. Enjoy!
            </div>
          </div>

          <div className="text-sm text-muted-foreground text-center mt-4">
            Currently logged in as {shortenAddress(address)}{" "}
            <button
              onClick={() => {
                disconnect();
              }}
              className="text-blue-500 underline"
            >
              Logout
            </button>
          </div>
        </div>
      </div>

      <Dialog open={isCreatingBoard} onOpenChange={onOpenChangeCreateBoard}>
        <DialogOverlay />
        <DialogContent
          className="w-[360px] rounded-3xl p-4 gap-0 pb-10"
          showCloseButton
        >
          {createBoardStep === CreateBoardStep.JOIN && (
            <div>
              <DialogTitle className="text-center font-medium text-base mb-6">
                Join or Create Game Session
              </DialogTitle>

              <DialogDescription className="text-sm text-muted-foreground text-center mb-6">
                Enter a session code to join a game or create a new one.
              </DialogDescription>

              <div className="bg-muted rounded-xl mb-6 relative">
                <input
                  value={joinCode}
                  onChange={(e) => setJoinCode(e.target.value)}
                  className="text-sm text-muted-foreground p-4 border-none outline-none"
                  placeholder="Session code"
                />

                <button
                  onClick={() =>
                    setJoinCode(Math.floor(Math.random() * 1000000).toString())
                  }
                  className="absolute right-0 top-1/2 -translate-y-1/2 w-10 flex items-center justify-center"
                >
                  <Shuffle className="w-4 h-4" />
                </button>
              </div>

              <div className="space-y-2">
                <Button
                  size="lg"
                  className="w-full"
                  onClick={() => onCreateBoard(BigInt(joinCode))}
                  disabled={!joinCode}
                >
                  Create Game
                </Button>
                <Button
                  size="lg"
                  className="w-full"
                  variant="secondary"
                  onClick={() => onJoinBoard(BigInt(joinCode))}
                  disabled={!joinCode}
                >
                  Join Game
                </Button>
              </div>
            </div>
          )}

          {createBoardStep === CreateBoardStep.PROOF && (
            <div className="flex flex-col items-center">
              <div className="w-10 h-10 mb-4 flex items-center justify-center bg-muted rounded-full">
                <AudioWaveform className="w-4 h-4" />
              </div>

              <div className="font-medium text-center mb-1">
                Generating proof...
              </div>

              <div className="text-xs text-muted-foreground text-center">
                This may take a few seconds.
              </div>
            </div>
          )}

          {createBoardStep === CreateBoardStep.SIGN_AND_SEND && (
            <div className="flex flex-col items-center">
              {connector?.icon ? (
                <img
                  src={connector?.icon}
                  alt={connector?.name}
                  className="w-10 h-10 mb-4"
                />
              ) : (
                <div className="w-10 h-10 mb-4 flex items-center justify-center bg-muted rounded-full">
                  <WalletIcon className="w-4 h-4" />
                </div>
              )}

              <div className="font-medium text-center mb-1">
                Waiting for {connector?.name ?? "wallet"}...
              </div>

              <div className="text-xs text-muted-foreground text-center">
                Don’t see your wallet? Check your other browser windows.
              </div>
            </div>
          )}

          {createBoardStep === CreateBoardStep.WAITING_JOIN && (
            <div className="flex flex-col items-center">
              <div className="w-10 h-10 mb-4 flex items-center justify-center bg-muted rounded-full">
                <CheckIcon className="w-4 h-4" />
              </div>

              <div className="font-medium text-center mb-1 max-w-[200px]">
                Waiting for opponent to join...
              </div>

              <div className="text-xs text-muted-foreground text-center">
                Share the session code {joinCode.toString()} with your opponent
                to start the game!
              </div>
            </div>
          )}

          {createBoardStep === CreateBoardStep.SUCCESS && (
            <div className="flex flex-col items-center">
              <div className="w-10 h-10 mb-4 flex items-center justify-center bg-muted rounded-full">
                <CheckIcon className="w-4 h-4" />
              </div>

              <div className="font-medium text-center mb-1">Board created!</div>

              <div className="text-xs text-muted-foreground text-center">
                Your're opponent already attacked, lets't respond!
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>

      <Dialog open={isAttacking} onOpenChange={onOpenChangeAttack}>
        <DialogOverlay />
        <DialogContent
          className="w-[360px] rounded-3xl p-4 gap-0 pb-10"
          showCloseButton
        >
          {attackStep === AttackStep.PROOF && (
            <div className="flex flex-col items-center">
              <div className="w-10 h-10 mb-4 flex items-center justify-center bg-muted rounded-full">
                <AudioWaveform className="w-4 h-4" />
              </div>

              <div className="font-medium text-center mb-1">
                Generating proof...
              </div>

              <div className="text-xs text-muted-foreground text-center">
                This may take a few seconds.
              </div>
            </div>
          )}

          {attackStep === AttackStep.SIGN_AND_SEND && (
            <div className="flex flex-col items-center">
              {connector?.icon ? (
                <img
                  src={connector?.icon}
                  alt={connector?.name}
                  className="w-10 h-10 mb-4"
                />
              ) : (
                <div className="w-10 h-10 mb-4 flex items-center justify-center bg-muted rounded-full">
                  <WalletIcon className="w-4 h-4" />
                </div>
              )}

              <div className="font-medium text-center mb-1">
                Waiting for {connector?.name ?? "wallet"}...
              </div>

              <div className="text-xs text-muted-foreground text-center">
                Don’t see your wallet? Check your other browser windows.
              </div>
            </div>
          )}

          {attackStep === AttackStep.WAITING_FOR_CONFIRMATION && (
            <div className="flex flex-col items-center">
              <div className="w-10 h-10 mb-4 flex items-center justify-center bg-muted rounded-full">
                <CheckIcon className="w-4 h-4" />
              </div>

              <div className="font-medium text-center mb-1 max-w-[200px]">
                Waiting for opponent to respond...
              </div>

              <div className="text-xs text-muted-foreground text-center">
                Be patient, this may take some time. You'll see the shot at the
                logs.
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </>
  );
}

export const GameLog: React.FC<{
  log: Log;
}> = ({ log }) => {
  const { address } = useAccount();
  const isUser = log.from === address;

  if (log.type === "create") {
    return (
      <div className="space-x-2 text-sm">
        {isUser ? "You" : shortenAddress(log.from)} created game at txn:{" "}
        <a
          className="text-blue-500 underline"
          href={`https://sepolia.arbiscan.io/tx/${log.txHash}`}
          target="_blank"
        >
          {shortenAddress(log.txHash)}
        </a>
      </div>
    );
  }

  if (log.type === "join") {
    return (
      <div className="space-x-2 text-sm">
        {isUser ? "You" : shortenAddress(log.from)} joined game at txn:{" "}
        <a
          className="text-blue-500 underline"
          href={`https://sepolia.arbiscan.io/tx/${log.txHash}`}
          target="_blank"
        >
          {shortenAddress(log.txHash)}
        </a>
      </div>
    );
  }

  if (log.type === "attack") {
    return (
      <div className="space-x-2 text-sm">
        {isUser ? "You" : shortenAddress(log.from)} attacked at{" "}
        {"ABCDEFGHIJ"[log.args.x]}
        {log.args.y} and {log.args.hit ? "hit" : "missed"} in txn:{" "}
        <a
          className="text-blue-500 underline"
          href={`https://sepolia.arbiscan.io/tx/${log.txHash}`}
          target="_blank"
        >
          {shortenAddress(log.txHash)}
        </a>
      </div>
    );
  }

  if (log.type === "lost") {
    return <div className="space-x-2 text-sm">Sorry, you lost the game...</div>;
  }

  if (log.type === "win") {
    return (
      <div className="space-x-2 text-sm">
        Congratulations, you won the game!
      </div>
    );
  }

  return null;
};

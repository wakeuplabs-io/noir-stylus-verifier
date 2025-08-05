import { createFileRoute, Navigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { BattleshipBoard, BattleshipBoardCell } from "@/components/board";
import {
  placeShipsRandomly,
  type CellState,
  type PlacementResult,
} from "@/lib/board";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import {
  ArrowLeftIcon,
  ArrowRightIcon,
  AudioWaveform,
  CheckIcon,
  WalletIcon,
} from "lucide-react";
import {
  DialogDescription,
  DialogOverlay,
  DialogTitle,
} from "@radix-ui/react-dialog";
import { useAccount } from "wagmi";
import { Button } from "@/components/ui/button";
import { shortenAddress } from "@/lib/utils";

export const Route = createFileRoute("/board")({
  component: RouteComponent,
});

type LogType = "position" | "commit" | "attack" | "turn" | "gameOver";

type Log = {
  type: LogType;
  from: string;
  message: string;
  txHash: string;
};

function RouteComponent() {
  const { address, connector } = useAccount();

  const [boardId, setBoardId] = useState<string>("");
  const [isCreatingBoard, setIsCreatingBoard] = useState<boolean>(false);
  const [isAttacking, setIsAttacking] = useState<boolean>(false);
  const [attackStep, setAttackStep] = useState<
    "proof" | "signAndSend" | "waiting" | "success"
  >("proof");
  const [createBoardStep, setCreateBoardStep] = useState<
    "join" | "proof" | "signAndSend" | "waiting" | "success"
  >("proof");
  const [currentBoard, setCurrentBoard] = useState<"user" | "opponent">("user");
  const [userBoard, setUserBoard] = useState<PlacementResult>(
    placeShipsRandomly()
  );
  const [opponentBoard, setOpponentBoard] = useState<CellState[][]>(
    Array.from({ length: 10 }, () => Array.from({ length: 10 }, () => "empty"))
  );
  const [logs, setLogs] = useState<Log[]>([]);

  const onJoinOrCreate = () => {
    setIsCreatingBoard(true);
    setCreateBoardStep("join");
  };

  const onCreateBoard = async () => {
    setIsCreatingBoard(true);
    setCreateBoardStep("proof");

    await new Promise((resolve) => setTimeout(resolve, 1000));
    setCreateBoardStep("signAndSend");

    await new Promise((resolve) => setTimeout(resolve, 1000));
    setCreateBoardStep("waiting");
    // TODO: wait for opponent to join

    await new Promise((resolve) => setTimeout(resolve, 1000));
    setCreateBoardStep("success");

    setCurrentBoard("opponent");
    setBoardId("1234"); // TODO:
    setLogs((prev) => [
      {
        from: "0xOpponent",
        message: "Attacked at E4 and missed!",
        txHash: "0x123",
        type: "attack",
      },
      {
        from: "0xOpponent",
        message: "Ships have been placed on the board!",
        txHash: "0x123",
        type: "commit",
      },
      {
        from: address!,
        message: "Ships have been placed on the board!",
        txHash: "0x123",
        type: "commit",
      },
      ...prev,
    ]);

    await new Promise((resolve) => setTimeout(resolve, 1000));
    setIsCreatingBoard(false);
  };

  const onJoinBoard = async () => {
    setIsCreatingBoard(true);
    setCreateBoardStep("join");
  };

  const onOpenChangeCreateBoard = (open: boolean) => {
    if (!open) {
      setIsCreatingBoard(false);
    }
  };

  const onAttack = async (row: number, col: number) => {
    window.alert("Not implemented");
    setIsAttacking(true);
    setAttackStep("proof");

    await new Promise((resolve) => setTimeout(resolve, 1000));
    setAttackStep("signAndSend");

    await new Promise((resolve) => setTimeout(resolve, 1000));
    setAttackStep("waiting");

    await new Promise((resolve) => setTimeout(resolve, 1000));
    setAttackStep("success");

    setUserBoard((prev) => {
      const newBoard = [...prev.board];
      newBoard[row][col] = "hit";
      return { ...prev, board: newBoard };
    });
    setOpponentBoard((prev) => {
      const newBoard = [...prev];
      newBoard[row][col] = "hit";
      return newBoard;
    });
    setLogs((prev) => [
      {
        from: address!,
        message: "Attacked at E4 and missed!",
        txHash: "0x123",
        type: "attack",
      },
      ...prev,
    ]);
  };

  const onOpenChangeAttack = (open: boolean) => {
    if (!open && attackStep !== "waiting") {
      setIsAttacking(false);
      setAttackStep("proof");
    }
  };

  useEffect(() => {
    if (!address) {
      return;
    }

    // TODO: try to recover board from local storage if not finished

    // if not found
    setLogs([
      {
        from: address,
        message: "Ships have been placed on the board!",
        txHash: "0x123",
        type: "position",
      },
    ]);
  }, []);

  const hasCommitted = logs.some((log) => log.type === "commit");

  if (!address) {
    return <Navigate to="/" />;
  }

  return (
    <>
      <div className="h-screen w-screen">
        <div className="max-w-4xl mx-auto ">
          {currentBoard === "user" ? (
            <div className="mb-14 mt-16 w-full flex justify-between items-center">
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
            <div className="mb-14 mt-16 w-full flex justify-between items-center">
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
                board={userBoard.board}
                ships={userBoard.ships}
                className="border-none shadow-none rounded-r-none rounded-none bg-player-1"
              />
            ) : (
              <BattleshipBoard
                board={opponentBoard}
                ships={[]}
                onCellClick={onAttack}
                className="border-none shadow-none rounded-r-none rounded-none bg-player-2"
              />
            )}

            <div className="flex-1 border-l-2 border-black p-8 relative">
              <div className="font-bold text-xl">Game Logs...</div>

              <div className="mt-4 space-y-2">
                {logs.map((log) => (
                  <GameLog key={log.txHash} log={log} />
                ))}
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
                  <BattleshipBoardCell
                    row={0}
                    col={0}
                    state="empty"
                    ships={[]}
                  />
                  <ArrowRightIcon className="w-4 h-4 text-muted-foreground" />
                  <BattleshipBoardCell
                    row={0}
                    col={0}
                    state="miss"
                    ships={[]}
                  />
                  <ArrowRightIcon className="w-4 h-4 text-muted-foreground" />
                  <BattleshipBoardCell row={0} col={0} state="hit" ships={[]} />
                </div>
              </div>
            </div>
          </div>

          <div className="max-w-6xl mx-auto border-2 border-black rounded-2xl shadow-[2px_2px_0px_#000] p-8 pt-14 relative">
            <div className="px-4 flex items-center h-[44px] font-bold absolute border-2 border-black rounded-full shadow-[2px_2px_0px_#000] bg-white top-0 -translate-y-1/2 left-8">
              <span>About</span>
            </div>

            <div className="font-medium">
              Lorem ipsum, dolor sit amet consectetur adipisicing elit. Expedita
              rerum ipsa sunt totam aspernatur reiciendis! Ipsum nihil, at
              maxime laudantium, nam fugiat nostrum aut totam omnis blanditiis,
              obcaecati dolorum sunt.
            </div>
          </div>
        </div>
      </div>

      <Dialog open={isCreatingBoard} onOpenChange={onOpenChangeCreateBoard}>
        <DialogOverlay />
        <DialogContent
          className="w-[360px] rounded-3xl p-4 gap-0 pb-10"
          showCloseButton
        >
          {createBoardStep === "join" && (
            <div>
              <DialogTitle className="text-center font-medium text-base mb-6">
                Join or Create Game Session
              </DialogTitle>

              <DialogDescription className="text-sm text-muted-foreground text-center mb-6">
                Enter a session code to join a game or create a new one.
              </DialogDescription>

              <div className="bg-muted rounded-xl mb-6">
                <input
                  className="text-sm text-muted-foreground p-4 border-none outline-none"
                  placeholder="Session code"
                />
              </div>

              <Button size="lg" className="w-full" onClick={onCreateBoard}>
                Create Game Session
              </Button>
            </div>
          )}

          {createBoardStep === "proof" && (
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

          {createBoardStep === "signAndSend" && (
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

          {createBoardStep === "waiting" && (
            <div className="flex flex-col items-center">
              <div className="w-10 h-10 mb-4 flex items-center justify-center bg-muted rounded-full">
                <CheckIcon className="w-4 h-4" />
              </div>

              <div className="font-medium text-center mb-1 max-w-[200px]">
                Waiting for opponent to join...
              </div>

              <div className="text-xs text-muted-foreground text-center">
                Share the session code 1234 with your opponent to start the
                game!
              </div>
            </div>
          )}

          {createBoardStep === "success" && (
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
          {attackStep === "proof" && (
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

          {attackStep === "signAndSend" && (
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

          {attackStep === "waiting" && (
            <div className="flex flex-col items-center">
              <div className="w-10 h-10 mb-4 flex items-center justify-center bg-muted rounded-full">
                <CheckIcon className="w-4 h-4" />
              </div>

              <div className="font-medium text-center mb-1 max-w-[200px]">
                Waiting for opponent to verify and attack...
              </div>

              <div className="text-xs text-muted-foreground text-center">
                Be patient, this may take a few seconds.
              </div>
            </div>
          )}

          {attackStep === "success" && (
            <div className="flex flex-col items-center">
              <div className="w-10 h-10 mb-4 flex items-center justify-center bg-muted rounded-full">
                <CheckIcon className="w-4 h-4" />
              </div>

              <div className="font-medium text-center mb-1">
                Opponent responded!
              </div>

              <div className="text-xs text-muted-foreground text-center">
                You sunk their ship at A1! And got hit at B2... Your turn now!
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
  if (log.type === "position") {
    return (
      <div className="text-sm">
        Ships have been placed on the board!{" "}
        <a href="/board" className="text-blue-500 underline">
          Shuffle Again
        </a>
      </div>
    );
  }

  if (log.type === "commit") {
    return (
      <div className="space-x-2 text-sm">
        {shortenAddress(log.from)} committed to board at txn:{" "}
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
        {shortenAddress(log.from)} attacked at E4 and missed in txn:{" "}
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

  if (log.type === "turn") {
    return <div>{log.message}</div>;
  }

  if (log.type === "gameOver") {
    return <div>{log.message}</div>;
  }

  return <div>{log.message}</div>;
};

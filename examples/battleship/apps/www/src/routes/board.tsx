import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { BattleshipBoard } from "@/components/board";
import { placeShipsRandomly, type PlacementResult } from "@/lib/board";

export const Route = createFileRoute("/board")({
  component: RouteComponent,
});

function RouteComponent() {
  const [board, setBoard] = useState<PlacementResult>(
    // Array.from({ length: 10 }, () => Array(10).fill("empty"))
    placeShipsRandomly()
  );

  const handleClick = (row: number, col: number) => {};

  // if (!authenticated) {
  //   return <Navigate to="/" />;
  // }

  return (
    <div className="h-screen w-screen py-20">
      <div className="max-w-6xl mx-auto flex justify-between items-center mb-12">
        <div className="">
          <div className="mb-4">
            <h1 className="text-2xl font-bold">Your Board</h1>
          </div>

          <BattleshipBoard board={board.board} ships={board.ships} onCellClick={handleClick} />
          {/* <button className="relative px-4 py-2 text-lg font-bold text-black bg-[#FF8577] border-3 border-black rounded-full shadow-[4px_4px_0px_#000] active:shadow-none active:translate-x-[6px] active:translate-y-[6px] transition-transform duration-100">
            Login <span className="ml-2">→</span>
          </button> */}
        </div>
        <div>
          <div className="flex justify-end mb-4">
            <h1 className="text-2xl font-bold">Your Opponent's Board</h1>
          </div>
          <BattleshipBoard
            board={board.board}
            ships={board.ships}
            onCellClick={handleClick}
            className="bg-[#FF8577]"
          />
        </div>
      </div>

      <div className="max-w-6xl mx-auto border-3 border-black rounded-2xl shadow-[2px_2px_0px_#000] p-6 relative">
        <div className="px-4 flex items-center h-10 font-bold absolute border-3 border-black rounded-full shadow-[2px_2px_0px_#000] bg-white top-0 -translate-y-1/2 left-4">
          <span>Logs</span>
     
        </div>
      </div>
    </div>
  );
}

import React from "react";
import clsx from "clsx";
import { cn } from "@/lib/utils";
import { CrossIcon, DotIcon, XIcon } from "lucide-react";
import type { ShipPlacement } from "@/lib/board";
import boatEnd from "@/assets/boat-end.svg";
import boatMiddle from "@/assets/boat-middle.svg";

const letters = "ABCDEFGHIJ".split("");

export type CellState = "empty" | "ship" | "hit" | "miss";

export const BattleshipBoard: React.FC<{
  board: CellState[][];
  ships: ShipPlacement[];
  onCellClick?: (row: number, col: number) => void;
  className?: string;
}> = ({ board, ships, onCellClick, className }) => {
  return (
    <div
      className={cn(
        "bg-[#699BF7] p-8 pl-2 rounded-2xl border-3 border-black shadow-[2px_2px_0px_#000]  space-y-2",
        className
      )}
    >
      {/* Header row */}
      <div className="grid grid-cols-11">
        <div></div>
        {Array.from({ length: 10 }, (_, i) => (
          <div key={i} className="text-center font-bold text-sm text-black">
            {letters[i]}
          </div>
        ))}
      </div>

      {/* Grid rows */}
      {board.map((row, rowIndex) => (
        <div key={rowIndex} className="grid grid-cols-11 gap-x-2">
          <div className="flex items-center justify-end pr-2 font-bold text-sm text-black">
            {rowIndex + 1}
          </div>
          {row.map((cell, colIndex) => (
            <div
              key={colIndex}
              className={clsx(
                "w-10 h-10 flex items-center justify-center cursor-pointer shrink-0 relative text-black  border-2 border-black rounded-lg shadow-[2px_2px_0px_#000] active:shadow-none active:translate-x-[2px] active:translate-y-[2px] transition-transform duration-100",
                {
                  "bg-white": cell === "ship" || cell === "empty" || cell === "miss",
                  "bg-[#F16858]": cell === "hit",
                }
              )}
              onClick={() => onCellClick?.(rowIndex, colIndex)}
            >
              {cell === "hit" && <XIcon className="w-6 h-6" />}
              {cell === "miss" && <DotIcon className="w-10 h-10" />}
              {cell === "ship" && getShipSegment(rowIndex, colIndex, ships)}
            </div>
          ))}
        </div>
      ))}
    </div>
  );
};

const getShipSegment = (
  row: number,
  col: number,
  shipPlacements: ShipPlacement[]
) => {
  for (const ship of shipPlacements) {
    const { start, direction, size } = ship;

    const dx = direction === "horizontal" ? 1 : 0;

    for (let i = 0; i < size; i++) {
      const segmentRow = start.row + (dx ? 0 : i);
      const segmentCol = start.col + (dx ? i : 0);

      if (segmentRow === row && segmentCol === col) {
        const isStart = i === 0;
        const isEnd = i === size - 1;
        const rotation =
          direction === "horizontal"
            ? isStart
              ? 180
              : isEnd
              ? 0
              : 0
            : isStart
            ? 270
            : isEnd
            ? 90
            : 90;

        return isStart || isEnd ? (
          <img
            src={boatEnd}
            className={cn("w-10 h-10")}
            style={{ transform: `rotate(${rotation + 180}deg)` }}
          />
        ) : (
          <img
            src={boatMiddle}
            className="w-10 h-10"
            style={{ transform: `rotate(${rotation}deg)` }}
          />
        );
      }
    }
  }

  return null;
};

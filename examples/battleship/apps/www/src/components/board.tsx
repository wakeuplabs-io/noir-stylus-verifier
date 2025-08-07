import React from "react";
import clsx from "clsx";
import { cn } from "@/lib/utils";
import { DotIcon, XIcon } from "lucide-react";
import boatEnd from "@/assets/boat-end.svg";
import boatMiddle from "@/assets/boat-middle.svg";
import {
  BoardCellState,
  Direction,
  SHIP_LENGTHS,
  ShipType,
  type BoardShips,
} from "@battleship/core";

const letters = "ABCDEFGHIJ".split("");

export const BattleshipBoard: React.FC<{
  board: BoardCellState[][];
  ships?: BoardShips;
  onCellClick?: (row: number, col: number) => void;
  className?: string;
}> = ({ board, ships, onCellClick, className }) => {
  return (
    <div
      className={cn(
        "p-8 pl-2 rounded-2xl border-2 border-black shadow-[2px_2px_0px_#000] space-y-2",
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
            <BattleshipBoardCell
              state={cell}
              ships={ships}
              x={colIndex}
              y={rowIndex}
              key={colIndex}
              onClick={() => onCellClick?.(rowIndex, colIndex)}
            />
          ))}
        </div>
      ))}
    </div>
  );
};

export const BattleshipBoardCell: React.FC<{
  x?: number;
  y?: number;
  state: BoardCellState;
  ships?: BoardShips;
  onClick?: () => void;
}> = ({ x = 0, y = 0, ships, state, onClick }) => {
  return (
    <div
      className={clsx(
        "w-10 h-10 flex items-center justify-center cursor-pointer shrink-0 relative text-black  border-2 border-black rounded-lg shadow-[2px_2px_0px_#000] active:shadow-none active:translate-x-[2px] active:translate-y-[2px] hover:shadow-none hover:translate-x-[2px] hover:translate-y-[2px] transition-transform duration-100",
        {
          "bg-white":
            state === BoardCellState.SHIP ||
            state === BoardCellState.EMPTY ||
            state === BoardCellState.MISS,
          "bg-[#F16858]": state === BoardCellState.HIT,
        }
      )}
      onClick={onClick}
    >
      {state === BoardCellState.HIT && <XIcon className="w-6 h-6" />}
      {state === BoardCellState.MISS && <DotIcon className="w-10 h-10" />}
      {state === BoardCellState.SHIP && ships && getShipSegment(x, y, ships)}
    </div>
  );
};

const getShipSegment = (x: number, y: number, ships: BoardShips) => {
  for (const shipType of Object.values(ShipType)) {
    const ship = ships[shipType];
    const size = SHIP_LENGTHS[shipType];

    const isStart = ship.x === x && ship.y === y;

    const endX =
      ship.direction === Direction.HORIZONTAL ? ship.x + size - 1 : ship.x;
    const endY =
      ship.direction === Direction.VERTICAL ? ship.y + size - 1 : ship.y;
    const isEnd = endX === x && endY === y;
    const isBody =
      ship.direction === Direction.HORIZONTAL
        ? x >= ship.x && x <= ship.x + size && y === ship.y
        : x === ship.x && y >= ship.y && y <= ship.y + size;

    if (isStart || isEnd) {
      let rotation = ship.direction === Direction.HORIZONTAL ? 0 : 90;
      if (isEnd) {
        rotation += 180;
      }

      return (
        <img
          src={boatEnd}
          className={cn("w-10 h-10")}
          style={{ transform: `rotate(${rotation}deg)` }}
        />
      );
    } else if (isBody) {
      const rotation = ship.direction === Direction.HORIZONTAL ? 0 : 90;
      return (
        <img
          src={boatMiddle}
          className="w-10 h-10"
          style={{ transform: `rotate(${rotation}deg)` }}
        />
      );
    }

  }

  return null;
};

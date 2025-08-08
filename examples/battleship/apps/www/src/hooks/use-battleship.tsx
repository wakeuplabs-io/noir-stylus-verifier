import { useState } from "react";
import { useAccount } from "wagmi";

export const useBattleship = () => {
  const [boardId, setBoardId] = useState<string>("");
  const [isAttacking, setIsAttacking] = useState<boolean>(false);
  const [attackStep, setAttackStep] = useState<
    "proof" | "signAndSend" | "waiting" | "success"
  >("proof");

  const onAttack = async (row: number, col: number) => {
  }

  const onCreateBoard = async (board: any) => {
  }

  return {
    boardId,
    isAttacking,
    attackStep,
  };
};

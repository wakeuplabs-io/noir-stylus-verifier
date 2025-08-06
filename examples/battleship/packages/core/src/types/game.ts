
export type Game = {
  player1: string;
  player2: string;
  player1BoardHash: string;
  player2BoardHash: string;
  joinCode: string;
  moves: Move[];
};

export type Move = {
    x: bigint;
    y: bigint;
    isHit: boolean;
  };
  
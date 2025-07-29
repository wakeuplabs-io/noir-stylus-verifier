import { poseidon2Hash } from "@zkpassport/poseidon2";

export class MerkleTree {
  private leaves: bigint[];
  private levels: bigint[][] = [];
  private nextIndex: number;
  private zeroLeaf: bigint;
  private depth: number;
  private maxLeaves: number;

  constructor(depth: number, zeroLeaf: bigint) {
    this.depth = depth;
    this.zeroLeaf = zeroLeaf;
    this.maxLeaves = 2 ** depth;
    this.leaves = new Array(this.maxLeaves).fill(zeroLeaf);
    this.nextIndex = 0;
  }

  async addCommitment(commitment: bigint) {
    if (this.nextIndex >= this.maxLeaves) {
      throw new Error("Merkle tree is full. No more leaves can be added.");
    }
    this.leaves[this.nextIndex] = commitment;
    this.nextIndex++;
    await this.recalculateTree();
  }

  private async recalculateTree() {
    this.levels = [[...this.leaves]];

    let currentLevel = this.levels[0];
    let size = currentLevel.length;

    while (size > 1) {
      const nextSize = Math.ceil(size / 2);
      const nextLevel: bigint[] = new Array(nextSize);

      for (let i = 0; i < nextSize; i++) {
        const left = currentLevel[2 * i] ?? this.zeroLeaf;
        const right = currentLevel[2 * i + 1] ?? left;

        nextLevel[i] = poseidon2Hash([left, right]);
      }

      this.levels.push(nextLevel);
      size = nextSize;
      currentLevel = nextLevel;
    }
  }

  getRoot(): bigint {
    if (this.levels.length === 0) {
      return this.zeroLeaf;
    }
    const topLevel = this.levels[this.levels.length - 1];
    return topLevel.length > 0 ? topLevel[0] : this.zeroLeaf;
  }

  getProof(commitment: bigint): {
    path: bigint[];
    directionSelector: boolean[];
  } {
    const index = this.leaves.indexOf(commitment);
    if (index === -1) return { path: [], directionSelector: [] };

    const path: bigint[] = [];
    const directionSelector: boolean[] = [];

    let currentIndex = index;

    for (let level = 0; level < this.depth; level++) {
      if (level >= this.levels.length - 1) break;

      const levelNodes = this.levels[level];
      const isRightNode = currentIndex % 2 === 1;

      const pairIndex = isRightNode ? currentIndex - 1 : currentIndex + 1;

      if (pairIndex < levelNodes.length) {
        path.push(levelNodes[pairIndex]);
      } else {
        path.push(this.zeroLeaf);
      }
      directionSelector.push(isRightNode);

      currentIndex = Math.floor(currentIndex / 2);
    }

    return { path, directionSelector };
  }
}

import {
  Address,
  Chain,
  createPublicClient,
  createWalletClient,
  http,
  PublicClient,
  WalletClient,
} from "viem";
import { VotingContractAbi } from "../config/abi";

export class VotingContract {
  private chain: Chain;
  private address: Address;
  private privateKey: `0x${string}` | undefined;
  private publicClient: PublicClient;
  private walletClient: WalletClient;

  constructor(
    address: Address,
    chain: Chain,
    rpcUrl: string,
    privateKey?: `0x${string}`
  ) {
    this.chain = chain;
    this.address = address;
    this.privateKey = privateKey;
    this.publicClient = createPublicClient({
      chain,
      transport: http(rpcUrl),
    });
    this.walletClient = createWalletClient({
      chain,
      transport: http(rpcUrl),
    });
  }

  async propose(description: string, deadline: bigint, votersRoot: bigint) {
    if (!this.privateKey) {
      throw new Error("Private key not found");
    }

    const tx = await this.walletClient.writeContract({
      address: this.address,
      abi: VotingContractAbi,
      functionName: "propose",
      args: [description, deadline, votersRoot],
      account: this.privateKey,
      chain: this.chain,
    });
    return tx;
  }

  async getProposal(proposalId: number) {
    const proposal = await this.publicClient.readContract({
      address: this.address,
      abi: VotingContractAbi,
      functionName: "getProposal",
      args: [proposalId],
    });
    return {
        description: proposal[0],
        deadline: proposal[1],
        forVotes: proposal[2],
        againstVotes: proposal[3],
        votersRoot: proposal[4],
    };
  }

  async castVote(proof: string, proposalId: number, vote: number, nullifierHash: bigint) {
    if (!this.privateKey) {
      throw new Error("Private key not found");
    }

    const tx = await this.walletClient.writeContract({
      address: this.address,
      abi: VotingContractAbi,
      functionName: "castVote",
      args: [proof, proposalId, vote, nullifierHash],
      account: this.privateKey,
      chain: this.chain,
    });
    return tx;
  }
}

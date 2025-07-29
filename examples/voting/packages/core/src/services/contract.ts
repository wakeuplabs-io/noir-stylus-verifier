import {
  Address,
  Chain,
  checksumAddress,
  createPublicClient,
  createWalletClient,
  decodeEventLog,
  encodeEventTopics,
  getAbiItem,
  http,
  PublicClient,
  WalletClient,
} from "viem";
import { VotingContractAbi } from "../config/abi";
import { privateKeyToAccount } from "viem/accounts";

export class VotingContract {
  private address: Address;
  private privateKey: `0x${string}` | undefined;
  private publicClient: PublicClient;
  private walletClient: WalletClient;

  constructor(address: Address, rpcUrl: string, privateKey?: `0x${string}`) {
    this.address = checksumAddress(address);
    this.privateKey = privateKey;
    this.publicClient = createPublicClient({
      transport: http(rpcUrl),
    });
    this.walletClient = createWalletClient({
      transport: http(rpcUrl),
    });
  }

  async propose(metadata: string, deadline: bigint, votersRoot: bigint) {
    if (!this.privateKey) {
      throw new Error("Private key not found");
    }

    try {
      const tx = await this.walletClient.writeContract({
        address: this.address,
        abi: VotingContractAbi,
        functionName: "propose",
        args: [metadata, deadline, votersRoot],
        account: privateKeyToAccount(this.privateKey),
        chain: null,
      });

      // extract the proposal id from the tx
      const receipt = await this.publicClient.waitForTransactionReceipt({
        hash: tx,
      });
      const [proposalCreatedTopic] = encodeEventTopics({
        abi: VotingContractAbi,
        eventName: "ProposalCreated",
      });

      // Find and decode the matching log
      const log = receipt.logs.find(
        (log) => log.topics[0] === proposalCreatedTopic
      );
      if (!log) throw new Error("Log not found");

      const decoded = decodeEventLog({
        abi: VotingContractAbi,
        data: log.data,
        topics: log.topics,
      });

      return { id: decoded.args.id, tx: tx };
    } catch (error) {
      console.error(error);
      throw error;
    }
  }

  async getProposalMetadata(proposalId: number) {
    return this.publicClient.readContract({
      address: this.address,
      abi: VotingContractAbi,
      functionName: "getProposalMetadata",
      args: [BigInt(proposalId)],
    });
  }

  async getProposal(proposalId: number) {
    const [metadata, deadline, votersRoot, forVotes, againstVotes] =
      await Promise.all([
        this.publicClient.readContract({
          address: this.address,
          abi: VotingContractAbi,
          functionName: "getProposalMetadata",
          args: [BigInt(proposalId)],
        }),
        this.publicClient.readContract({
          address: this.address,
          abi: VotingContractAbi,
          functionName: "getProposalDeadline",
          args: [BigInt(proposalId)],
        }),
        this.publicClient.readContract({
          address: this.address,
          abi: VotingContractAbi,
          functionName: "getProposalVotersRoot",
          args: [BigInt(proposalId)],
        }),
        this.publicClient.readContract({
          address: this.address,
          abi: VotingContractAbi,
          functionName: "getProposalForVotes",
          args: [BigInt(proposalId)],
        }),
        this.publicClient.readContract({
          address: this.address,
          abi: VotingContractAbi,
          functionName: "getProposalAgainstVotes",
          args: [BigInt(proposalId)],
        }),
      ]);

    return {
      metadata: metadata,
      deadline: deadline,
      forVotes: forVotes,
      againstVotes: againstVotes,
      votersRoot: votersRoot,
    };
  }

  async castVote(
    proof: `0x${string}`,
    proposalId: number,
    vote: number,
    nullifierHash: bigint
  ) {
    if (!this.privateKey) {
      throw new Error("Private key not found");
    }

    const tx = await this.walletClient.writeContract({
      address: this.address,
      abi: VotingContractAbi,
      functionName: "castVote",
      args: [proof, BigInt(proposalId), BigInt(vote), nullifierHash],
      account: privateKeyToAccount(this.privateKey),
      chain: null,
    });
    return tx;
  }
}

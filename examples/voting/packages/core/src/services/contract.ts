import {
  Address,
  checksumAddress,
  createPublicClient,
  decodeEventLog,
  encodeEventTopics,
  encodeFunctionData,
  http,
  PublicClient,
} from "viem";
import { VotingContractAbi } from "../config/abi";
import { Proposal, ProposalMetadata } from "../types/proposal";
import { MULTICALL_ADDRESS, SupportedChainId } from "../config/constants";
import { IpfsClient } from "../infrastructure/ipfs";

export class VotingContract {
  private address: Address;
  private chainId: SupportedChainId;
  private publicClient: PublicClient;
  private ipfsClient: IpfsClient;

  constructor(
    ipfsClient: IpfsClient,
    chainId: SupportedChainId,
    address: `0x${string}`,
    rpcUrl: string
  ) {
    this.address = checksumAddress(address);
    this.chainId = chainId;
    this.ipfsClient = ipfsClient;

    this.publicClient = createPublicClient({
      transport: http(rpcUrl),
    });
  }

  async isNullifierUsed(nullifierHash: bigint): Promise<boolean> {
    return this.publicClient.readContract({
      address: this.address,
      abi: VotingContractAbi,
      functionName: "isNullifierUsed",
      args: [nullifierHash],
    });
  }

  async preparePropose(
    userAddress: `0x${string}`,
    metadata: ProposalMetadata,
    deadline: bigint,
    votersRoot: bigint
  ) {
    const metadataCid = await this.ipfsClient.uploadJSON(metadata);

    const txRequest = await this.publicClient.prepareTransactionRequest({
      to: this.address,
      data: encodeFunctionData({
        abi: VotingContractAbi,
        functionName: "propose",
        args: [metadataCid, deadline, votersRoot],
      }),
      value: 0n,
      chain: null,
      account: userAddress,
    });

    return txRequest;
  }

  async recoverProposalId(hash: `0x${string}`): Promise<number> {
    // extract the proposal id from the tx
    const receipt = await this.publicClient.waitForTransactionReceipt({
      hash: hash,
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

    return (decoded.args as any).id;
  }

  async getProposalMetadata(proposalId: number): Promise<string> {
    return this.publicClient.readContract({
      address: this.address,
      abi: VotingContractAbi,
      functionName: "getProposalMetadata",
      args: [BigInt(proposalId)],
    });
  }

  async getProposalCount(): Promise<bigint> {
    return this.publicClient.readContract({
      address: this.address,
      abi: VotingContractAbi,
      functionName: "getProposalCount",
    });
  }

  async getProposalDeadline(proposalId: number): Promise<bigint> {
    return this.publicClient.readContract({
      address: this.address,
      abi: VotingContractAbi,
      functionName: "getProposalDeadline",
      args: [BigInt(proposalId)],
    });
  }

  async getProposalForVotes(proposalId: number): Promise<bigint> {
    return this.publicClient.readContract({
      address: this.address,
      abi: VotingContractAbi,
      functionName: "getProposalForVotes",
      args: [BigInt(proposalId)],
    });
  }

  async getProposalAgainstVotes(proposalId: number): Promise<bigint> {
    return this.publicClient.readContract({
      address: this.address,
      abi: VotingContractAbi,
      functionName: "getProposalAgainstVotes",
      args: [BigInt(proposalId)],
    });
  }

  async getProposalCreatedAt(proposalId: number): Promise<bigint> {
    return this.publicClient.readContract({
      address: this.address,
      abi: VotingContractAbi,
      functionName: "getProposalCreatedAt",
      args: [BigInt(proposalId)],
    });
  }

  async getProposalAuthor(proposalId: number): Promise<string> {
    return this.publicClient.readContract({
      address: this.address,
      abi: VotingContractAbi,
      functionName: "getProposalAuthor",
      args: [BigInt(proposalId)],
    });
  }

  async getProposal(proposalId: number): Promise<Proposal> {
    const [root, metadataCid, deadline, forVotes, againstVotes, createdAt, author] =
      await this.publicClient
        .multicall({
          multicallAddress: MULTICALL_ADDRESS[this.chainId],
          contracts: [
            {
              address: this.address,
              abi: VotingContractAbi,
              functionName: "getProposalVotersRoot",
              args: [BigInt(proposalId)],
            },
            {
              address: this.address,
              abi: VotingContractAbi,
              functionName: "getProposalMetadata",
              args: [BigInt(proposalId)],
            },
            {
              address: this.address,
              abi: VotingContractAbi,
              functionName: "getProposalDeadline",
              args: [BigInt(proposalId)],
            },
            {
              address: this.address,
              abi: VotingContractAbi,
              functionName: "getProposalForVotes",
              args: [BigInt(proposalId)],
            },
            {
              address: this.address,
              abi: VotingContractAbi,
              functionName: "getProposalAgainstVotes",
              args: [BigInt(proposalId)],
            },
            {
              address: this.address,
              abi: VotingContractAbi,
              functionName: "getProposalCreatedAt",
              args: [BigInt(proposalId)],
            },
            {
              address: this.address,
              abi: VotingContractAbi,
              functionName: "getProposalAuthor",
              args: [BigInt(proposalId)],
            },
          ],
        })
        .then((results) =>
          results.map((r) => {
            if (r.error) throw new Error(r.error.message);
            return r.result;
          })
        );

    const metadata = (await this.ipfsClient.downloadJSON(
      metadataCid as string
    )) as ProposalMetadata;

    return {
      id: proposalId,
      root: root.toString(),
      metadata,
      deadline: new Date(Number(deadline) * 1000),
      for: Number(forVotes),
      against: Number(againstVotes),
      createdAt: new Date(Number(createdAt) * 1000),
      author: author as string,
      status:
        new Date(Number(deadline) * 1000) > new Date()
          ? "active"
          : forVotes > againstVotes
          ? "passed"
          : "rejected",
    };
  }

  async prepareCastVote(
    userAddress: `0x${string}`,
    proof: `0x${string}`,
    proposalId: number,
    vote: boolean,
    nullifier: bigint
  ) {
    const txRequest = await this.publicClient.prepareTransactionRequest({
      to: this.address,
      data: encodeFunctionData({
        abi: VotingContractAbi,
        functionName: "castVote",
        args: [proof, BigInt(proposalId), BigInt(vote ? 1 : 0), nullifier],
      }),
      value: 0n,
      chain: null,
      account: userAddress,
    });

    return txRequest;
  }
}

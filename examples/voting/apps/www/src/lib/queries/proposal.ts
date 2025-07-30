import { IPFS_GATEWAY_URL, IPFS_PINATA_JWT } from "@/env";
import { useInfiniteQuery, useQuery, useMutation } from "@tanstack/react-query";
import {
  DEPTH,
  MerkleTree,
  PinataIpfs,
  SupportedChainId,
  VotingContract,
  ZERO_LEAF,
  type ProposalMetadata,
} from "@voting/core";
import { useAccount, useSendTransaction } from "wagmi";

const contract = new VotingContract(
  SupportedChainId.ARBITRUM_SEPOLIA,
  new PinataIpfs(IPFS_PINATA_JWT, IPFS_GATEWAY_URL),
  undefined, // default contract address
  undefined // default rpc url
);
const PAGE_SIZE = 10;

class QueryKeyFactory {
  static proposal(proposalId: number) {
    return ["proposal", proposalId];
  }

  static proposalCount() {
    return ["proposal-count"];
  }

  static proposals() {
    return ["proposals"];
  }
}

export const useCreateProposal = () => {
  const { address } = useAccount();
  const { sendTransactionAsync } = useSendTransaction();
  return useMutation({
    mutationFn: async (metadata: ProposalMetadata) => {
      if (!address) {
        throw new Error("No account connected");
      }

      // Build the voters tree
      const votersTree = new MerkleTree(DEPTH, ZERO_LEAF);
      for (const voter of metadata.voters) {
        await votersTree.addCommitment(BigInt(voter));
      }
      const root = await votersTree.getRoot();

      // Prepare the transaction
      const txRequest = await contract.preparePropose(
        address,
        metadata,
        BigInt(Math.floor(metadata.deadline.getTime() / 1000)),
        root
      );

      // Send the transaction
      const tx = await sendTransactionAsync(txRequest);

      // Recover the proposal id
      const proposalId = await contract.recoverProposalId(tx);
      return proposalId;
    },
  });
};

export const useProposal = (proposalId: number) => {
  return useQuery({
    queryKey: QueryKeyFactory.proposal(proposalId),
    queryFn: () => contract.getProposal(proposalId),
  });
};

export const useProposalCount = () => {
  return useQuery({
    queryKey: QueryKeyFactory.proposalCount(),
    queryFn: () => contract.getProposalCount(),
  });
};

export const useProposals = () => {
  return useInfiniteQuery({
    queryKey: QueryKeyFactory.proposals(),
    queryFn: async ({ pageParam }) => {
      const count = await contract.getProposalCount();

      const pageStart = Math.max(Number(count) - pageParam * PAGE_SIZE - 1, 0);
      const pageEnd = Math.min(pageStart + PAGE_SIZE, Number(count));

      const proposals = await Promise.all(
        Array.from(
          { length: pageEnd - pageStart },
          (_, i) => pageStart + i
        ).map((id) => contract.getProposal(id))
      );
      return {
        proposals,
        hasNextPage: pageParam + PAGE_SIZE < count,
        nextPage: pageParam + PAGE_SIZE,
      };
    },
    getNextPageParam: (lastPage) => {
      return lastPage.hasNextPage ? lastPage.nextPage : undefined;
    },
    initialPageParam: 0,
  });
};

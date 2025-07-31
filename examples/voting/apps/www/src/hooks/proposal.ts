import {
  useInfiniteQuery,
  useQuery,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query";
import {
  DEPTH,
  MerkleTree,
  PinataIpfs,
  SupportedChainId,
  VotingCircuit,
  VotingContract,
  ZERO_LEAF,
  type ProposalMetadata,
} from "@voting/core";
import {
  useAccount,
  useSendTransaction,
  useAccount as useEvmAccount,
} from "wagmi";
import { useState, useCallback } from "react";
import { QueryKeyFactory } from "@/lib/queries";
import { IPFS_GATEWAY_URL, IPFS_PINATA_JWT } from "@/env";
import { useZkAccount } from "./account";

const PAGE_SIZE = 10;

const votingContract = new VotingContract(
  SupportedChainId.ARBITRUM_SEPOLIA,
  new PinataIpfs(IPFS_PINATA_JWT, IPFS_GATEWAY_URL),
  undefined, // default contract address
  undefined // default rpc url
);

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
      const txRequest = await votingContract.preparePropose(
        address,
        metadata,
        BigInt(Math.floor(metadata.deadline.getTime() / 1000)),
        root
      );

      // Send the transaction
      const tx = await sendTransactionAsync(txRequest);

      // TODO: invalidate proposal count query
      // TODO: invalidate proposals query

      // Recover the proposal id
      const proposalId = await votingContract.recoverProposalId(tx);
      return proposalId;
    },
  });
};

export const useProposalCount = () => {
  return useQuery({
    queryKey: QueryKeyFactory.proposalCount(),
    queryFn: () => votingContract.getProposalCount(),
  });
};

export const useProposals = () => {
  return useInfiniteQuery({
    queryKey: QueryKeyFactory.proposals(),
    queryFn: async ({ pageParam }) => {
      const count = await votingContract.getProposalCount();

      const pageStart = Math.max(
        Number(count) - (pageParam + 1) * PAGE_SIZE - 1,
        0
      );
      const pageEnd = Math.min(pageStart + PAGE_SIZE, Number(count));

      const proposals = await Promise.all(
        Array.from(
          { length: pageEnd - pageStart },
          (_, i) => pageStart + i
        ).map((id) => votingContract.getProposal(id))
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

export function useProposal(proposalId?: string | number) {
  const { account } = useZkAccount();

  return useQuery({
    enabled: !!proposalId, // only depends on proposalId
    queryKey: QueryKeyFactory.proposal(proposalId, account?.address ?? "0x0"),
    queryFn: async () => {
      const proposal = await votingContract.getProposal(Number(proposalId));
      const isEligible = proposal.metadata.voters.some(
        (voter: string) => BigInt(voter) === BigInt(account?.address ?? "0x0")
      );

      if (!account || !isEligible) {
        return { proposal, isEligible: false, alreadyVoted: false };
      }

      const nullifier = await VotingCircuit.generateNullifier(
        BigInt(proposal.root),
        BigInt(account.privateKey),
        BigInt(proposal.id)
      );

      const alreadyVoted = await votingContract.isNullifierUsed(nullifier);

      return {
        proposal,
        isEligible: true,
        alreadyVoted,
      };
    },
  });
}

export const useCastVote = (proposalId: number) => {
  const queryClient = useQueryClient();
  const { account } = useZkAccount();
  const { address: evmAddress } = useEvmAccount();
  const { sendTransactionAsync } = useSendTransaction();
  const { data: { proposal, isEligible, alreadyVoted } = {} } = useProposal(proposalId);

  const [isGeneratingProof, setIsGeneratingProof] = useState(false);
  const [isSendingTransaction, setIsSendingTransaction] = useState(false);

  const castVote = useCallback(
    async (vote: boolean) => {
      try {
        if (!evmAddress || !account?.address || !account?.privateKey) {
          throw new Error("Not connected");
        }
        if (!proposal) {
          throw new Error("Proposal not found");
        }
        if (!isEligible) {
          throw new Error("Not eligible to vote");
        }
        if (alreadyVoted) {
          throw new Error("Already voted");
        }

        setIsGeneratingProof(true);

        const nullifier = await VotingCircuit.generateNullifier(
          BigInt(proposal.root),
          BigInt(account.privateKey),
          BigInt(proposal.id)
        );

        const votersMerkleTree = new MerkleTree(
          DEPTH,
          ZERO_LEAF,
          proposal.metadata.voters.map(BigInt)
        );
        const { path, directionSelector } = votersMerkleTree.getProof(
          BigInt(account.address)
        );

        const proof = await VotingCircuit.generateProof(
          BigInt(proposalId),
          vote,
          BigInt(proposal.root),
          path,
          directionSelector,
          BigInt(account.privateKey),
          BigInt(account.secret),
          nullifier
        );

        setIsGeneratingProof(false);
        setIsSendingTransaction(true);

        const txRequest = await votingContract.prepareCastVote(
          evmAddress,
          `0x${Array.from(proof, (byte) =>
            byte.toString(16).padStart(2, "0")
          ).join("")}`,
          proposalId,
          vote,
          nullifier
        );
        const tx = await sendTransactionAsync(txRequest);

        // Refetch the available votes and proposal queries to update the UI
        await queryClient.refetchQueries({
          queryKey: QueryKeyFactory.proposal(proposalId, account?.address),
        });

        return tx;
      } catch (error) {
        console.error(error);
        throw error;
      } finally {
        setIsGeneratingProof(false);
        setIsSendingTransaction(false);
      }
    },
    [evmAddress, account, proposalId, proposal, isEligible, alreadyVoted]
  );

  return { castVote, disabled: !isEligible || alreadyVoted, isGeneratingProof, isSendingTransaction };
};

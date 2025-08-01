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
  VotingCircuit,
  VotingContract,
  ZERO_LEAF,
  type ProposalMetadata,
} from "@voting/core";
import { useAccount, useAccount as useEvmAccount } from "wagmi";
import { useState, useCallback, useEffect } from "react";
import { QueryKeyFactory } from "@/lib/queries";
import {
  IPFS_GATEWAY_URL,
  IPFS_PINATA_JWT,
  CONTRACT_ADDRESS,
  RPC_URL,
  CHAIN_ID,
} from "@/env";
import { useZkAccount } from "./account";
import { useChainId, useSendTransaction } from "wagmi";

const PAGE_SIZE = 10;

const votingContract = new VotingContract(
  new PinataIpfs(IPFS_PINATA_JWT, IPFS_GATEWAY_URL),
  CHAIN_ID,
  CONTRACT_ADDRESS,
  RPC_URL
);

export const useCreateProposal = () => {
  const chainId = useChainId();
  const { address } = useAccount();
  const queryClient = useQueryClient();
  const { sendTransactionAsync } = useSendTransaction();

  return useMutation({
    mutationFn: async (metadata: ProposalMetadata) => {
      // We need an evm wallet to send the transaction, and we need to be on the correct chain
      if (!address) {
        throw new Error("No EVM account connected");
      }

      // Some wallets like Metamask don't support chain switching, so we ask the user to do so manually
      if (BigInt(chainId) !== BigInt(CHAIN_ID)) {
        throw new Error(`Wrong chain, please switch to chain ${CHAIN_ID}`);
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

      await Promise.all([
        queryClient.refetchQueries({
          queryKey: QueryKeyFactory.proposalCount(),
        }),
        queryClient.refetchQueries({
          queryKey: QueryKeyFactory.proposals(),
        }),
      ]);

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
    enabled: proposalId != undefined, // only depends on proposalId
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
  const chainId = useChainId();
  const { account } = useZkAccount();
  const { address: evmAddress } = useEvmAccount();
  const { sendTransactionAsync } = useSendTransaction();
  const { data: { proposal, isEligible, alreadyVoted } = {} } =
    useProposal(proposalId);

  const [isGeneratingProof, setIsGeneratingProof] = useState(false);
  const [isSendingTransaction, setIsSendingTransaction] = useState(false);

  const castVote = useCallback(
    async (vote: boolean) => {
      try {
        // We need an evm wallet to send the transaction, and we need a zk account to generate the proof
        if (!evmAddress || !account?.address || !account?.privateKey) {
          throw new Error("Not connected");
        }

        // Some wallets like Metamask don't support chain switching, so we ask the user to do so manually
        if (BigInt(chainId) !== BigInt(CHAIN_ID)) {
          throw new Error(`Wrong chain, please switch to chain ${CHAIN_ID}`);
        }

        // Check if the proposal exists
        if (!proposal) {
          throw new Error("Proposal not found");
        }

        // Check if the user is eligible to vote
        if (!isEligible) {
          throw new Error("Not eligible to vote");
        }

        // Check if the user has already voted
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

  return {
    castVote,
    disabled: !isEligible || alreadyVoted || proposal?.status !== "active",
    isGeneratingProof,
    isSendingTransaction,
  };
};

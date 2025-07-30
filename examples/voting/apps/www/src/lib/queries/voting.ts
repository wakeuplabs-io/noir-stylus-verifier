import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useZkAccount } from "./account";
import {
  PinataIpfs,
  SupportedChainId,
  VotingCircuit,
  VotingContract,
  MerkleTree,
  DEPTH,
  ZERO_LEAF,
} from "@voting/core";
import { IPFS_GATEWAY_URL, IPFS_PINATA_JWT } from "@/env";
import { useAccount, useSendTransaction } from "wagmi";

class QueryKeyFactory {
  static availableVotes = (
    proposalId: number,
    accountAddress: `0x${string}`
  ) => ["voting-power", proposalId, accountAddress];
  static isNullifierUsed = (proposalId: number) => [
    "is-nullifier-used",
    proposalId,
  ];
}

const contract = new VotingContract(
  SupportedChainId.ARBITRUM_SEPOLIA,
  new PinataIpfs(IPFS_PINATA_JWT, IPFS_GATEWAY_URL),
  undefined, // default contract address
  undefined // default rpc url
);

export const useVotingPower = (proposalId: number) => {
  const { data: zkAccount } = useZkAccount();
  return useQuery({
    queryKey: QueryKeyFactory.availableVotes(
      proposalId,
      zkAccount.address ?? "0x0000000000000000000000000000000000000000"
    ),
    queryFn: async () => {
      if (!zkAccount.address) return 0;

      // TODO: optimize here to avoid fetching the proposal every time
      const proposal = await contract.getProposal(proposalId);
      if (
        !proposal.metadata.voters.find(
          (voter) => BigInt(voter) === BigInt(zkAccount.address ?? "")
        )
      ) {
        return 0;
      }

      const nullifier = await VotingCircuit.generateNullifier(
        BigInt(proposal.root),
        BigInt(zkAccount.privateKey),
        BigInt(proposal.id)
      );

      const isUsed = await contract.isNullifierUsed(nullifier);
      return isUsed ? 0 : 1;
    },
  });
};

export const useCastVote = (proposalId: number) => {
  const { address } = useAccount();
  const { data: zkAccount } = useZkAccount();
  const { sendTransactionAsync } = useSendTransaction();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (vote: boolean) => {
      if (!address || !zkAccount.address || !zkAccount.privateKey) {
        throw new Error("Not connected");
      }

      const proposal = await contract.getProposal(proposalId);

      const nullifier = await VotingCircuit.generateNullifier(
        BigInt(proposal.root),
        BigInt(zkAccount.privateKey),
        BigInt(proposal.id)
      );

      const tree = new MerkleTree(DEPTH, ZERO_LEAF);
      for (const voter of proposal.metadata.voters) {
        tree.addCommitment(BigInt(voter));
      }
      const { path, directionSelector } = tree.getProof(
        BigInt(zkAccount.address)
      );

      const proof = await VotingCircuit.generateProof(
        BigInt(proposalId),
        vote,
        BigInt(proposal.root),
        path,
        directionSelector,
        BigInt(zkAccount.privateKey),
        BigInt(zkAccount.secret),
        nullifier
      );

      const txRequest = await contract.prepareCastVote(
        address,
        `0x${Array.from(proof, (byte) =>
          byte.toString(16).padStart(2, "0")
        ).join("")}`,
        proposalId,
        vote,
        nullifier
      );
      const tx = await sendTransactionAsync(txRequest);
      return tx;
    },
    onSuccess: () => {
      queryClient.setQueryData(
        QueryKeyFactory.availableVotes(proposalId, zkAccount.address!),
        0
      );
    },
  });
};

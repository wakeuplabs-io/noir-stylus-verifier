import { parseAbi } from "viem";

export const VotingContractAbi = parseAbi([
  "function getVerifier() external view returns (address)",
  "function getProposalCount() external view returns (uint256)",
  "function isNullifierUsed(uint256 nullifier_hash) external view returns (bool)",
  "function propose(string calldata metadata, uint256 deadline, uint256 voters_root) external",
  "function getProposalMetadata(uint256 proposal_id) external view returns (string memory)",
  "function getProposalDeadline(uint256 proposal_id) external view returns (uint256)",
  "function getProposalForVotes(uint256 proposal_id) external view returns (uint256)",
  "function getProposalAgainstVotes(uint256 proposal_id) external view returns (uint256)",
  "function getProposalVotersRoot(uint256 proposal_id) external view returns (uint256)",
  "function getProposalAuthor(uint256 proposal_id) external view returns (address)",
  "function getProposalCreatedAt(uint256 proposal_id) external view returns (uint256)",
  "function castVote(bytes calldata proof, uint256 proposal_id, uint256 vote, uint256 nullifier_hash) external returns (bool)",
  "event ProposalCreated(uint256 indexed id)",
  "event NullifierUsed(uint256 indexed nullifier_hash)",
] as const);

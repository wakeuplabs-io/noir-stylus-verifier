import { parseAbi } from "viem";

export const VotingContractAbi = parseAbi([
  "function getVerifier() external view returns (address)",
  "function getProposal(uint256 proposal_id) external view returns (string memory, uint256, uint256, uint256, uint256)",
  "function getProposalDescription(uint256 proposal_id) external view returns (string memory)",
  "function getProposalDeadline(uint256 proposal_id) external view returns (uint256)",
  "function getProposalVotersRoot(uint256 proposal_id) external view returns (uint256)",
  "function getProposalForVotes(uint256 proposal_id) external view returns (uint256)",
  "function getProposalAgainstVotes(uint256 proposal_id) external view returns (uint256)",
  "function propose(string calldata description, uint256 deadline, uint256 voters_root) external",
  "function castVote(bytes calldata proof, uint256 proposal_id, uint256 vote, uint256 nullifier_hash) external returns (bool)",
  "event ProposalCreated(uint256 indexed proposal_id, string description, uint256 deadline, uint256 voters_root)"
] as const);

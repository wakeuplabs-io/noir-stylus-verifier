export const VotingContractAbi = [
  "function getVerifier() external view returns (address)",
  "function getProposal(uint256 proposal_id) external view returns (string memory, uint256, uint256, uint256, uint256)",
  "function propose(string calldata description, uint256 deadline, uint256 voters_root) external returns (uint256)",
  "function castVote(bytes calldata proof, uint256 proposal_id, uint256 vote, uint256 nullifier_hash) external returns (bool)",
  "function callVerify(bytes calldata proof, uint256 proposal_root, uint256 proposal_id, uint256 vote, uint256 nullifier_hash) external view returns (bool)",
] as const;

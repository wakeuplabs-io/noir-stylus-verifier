#!/usr/bin/env tsx

import fs from "fs";
import { Command } from "commander";
import { MerkleTree } from "./lib/merkle-tree";
import { VotingContract } from "./lib/contract";
import { arbitrumSepolia } from "viem/chains";
// import { UltraHonkBackend } from "@aztec/bb.js";
import { Noir } from "@noir-lang/noir_js";
import { ZkAccount } from "./lib/account";
import { poseidon2Hash } from "@zkpassport/poseidon2";

const program = new Command();

const DEPTH = 12;
const ZERO_LEAF = BigInt(0);
const CHAIN = arbitrumSepolia;
const CONTRACT_ADDRESS = "0x0000000000000000000000000000000000000000";
const DEFAULT_RPC_URL = "https://sepolia-rollup.arbitrum.io/rpc";
const ACCOUNT_MESSAGE = "Create ZK Account";

program.name("cli").description("Cli for voting management").version("0.1.0");

program
  .command("account") 
  .option("-p, --private-key <private-key>", "Private key")
  .description("Build a zk account")
  .action(async (options) => {
    const zkAccount = await ZkAccount.buildFromPrivateKey(options.privateKey, ACCOUNT_MESSAGE);
    
    console.log("Commitment:", "0x" + zkAccount.commitment.toString(16));
    console.log("Private key:", "0x" + zkAccount.priv_key.toString(16));
    console.log("Secret:", "0x" + zkAccount.secret.toString(16));
  });

program
  .command("propose")
  .requiredOption("-v, --voters <voters>", "File containing the voters")
  .requiredOption("-d, --description <description>", "Description is required")
  .requiredOption("-t, --deadline <deadline>", "Deadline is required")
  .requiredOption("-p, --private-key <private-key>", "Private key")
  .option("-r, --rpc-url <rpc-url>", "RPC URL")
  .description("Propose a new proposal for voting")
  .action(async (options) => {
    // build the merkle tree from the voters
    const voters = JSON.parse(fs.readFileSync(options.voters, "utf8"));
    const tree = new MerkleTree(DEPTH, ZERO_LEAF);
    for (const voter of voters) {
      await tree.addCommitment(BigInt(voter));
      console.log(`Added commitment for voter ${voter}`);
    }
    const root = await tree.getRoot();

    console.log(`Proposing a new proposal with root: 0x${root.toString(16)}`);
    const contract = new VotingContract(
      CONTRACT_ADDRESS,
      CHAIN,
      options.rpcUrl,
      options.privateKey
    );
    const tx = await contract.propose(
      options.description,
      options.deadline,
      root
    );
    console.log(`Proposal created at ${tx}`);
  });

program
  .command("get-proposal <proposal-id>")
  .option("-r, --rpc-url <rpc-url>", "RPC URL", DEFAULT_RPC_URL)
  .description("Get a proposal")
  .action(async (proposalId, options) => {
    const contract = new VotingContract(
      CONTRACT_ADDRESS,
      CHAIN,
      options.rpcUrl
    );
    const proposal = await contract.getProposal(proposalId);

    console.log(`Proposal ${proposalId} details:`);
    console.log(`Description: ${proposal.description}`);
    console.log(`Deadline: ${proposal.deadline}`);
    console.log(`Root: ${proposal.votersRoot}`);
    console.log(`For Votes: ${proposal.forVotes}`);
    console.log(`Against Votes: ${proposal.againstVotes}`);
  });

program
  .command("cast-vote <proposal-id> <vote>")
  .option("-r, --rpc-url <rpc-url>", "RPC URL", DEFAULT_RPC_URL)
  .option("-v, --voters <voters>", "File containing the voters")
  .option("-p, --private-key <private-key>", "Private key")
  .description("Cast a vote for a proposal")
  .action(async (proposalId, vote, options) => {
    console.log(
      `👋 Casting vote for proposal ${proposalId} with vote ${vote} from ${options.rpcUrl}`
    );

    const voters = JSON.parse(fs.readFileSync(options.voters, "utf8"));

    // derive secret and private key for user
    const zkAccount = await ZkAccount.buildFromPrivateKey(options.privateKey, ACCOUNT_MESSAGE);

    // rebuild the tree
    const tree = new MerkleTree(DEPTH, ZERO_LEAF);
    for (const voter of voters) {
      await tree.addCommitment(BigInt(voter));
    }
    const root = await tree.getRoot();
    const { path, directionSelector } = await tree.getProof(BigInt(voters[0]));

    // build proof
    const circuit = JSON.parse(
      fs.readFileSync("../circuits/contracts/assets/bytecode.json", "utf8")
    );
    const noir = new Noir(circuit);
    const backend = new UltraHonkBackend(circuit.bytecode);
    const { witness } = await noir.execute({
      root: root.toString(16),
      path: path.map((p) => p.toString(16)),
      directionSelector,
      secret: zkAccount.secret.toString(16),
      priv_key: zkAccount.priv_key.toString(16),
      nullifier: zkAccount.commitment.toString(16),
      proposal_id: proposalId.toString(16),
      vote: vote,
    });
    const { proof } = await backend.generateProof(witness, {
      keccak: true,
    });

    // build nullifier hash
    const nullifierHash = poseidon2Hash([root, zkAccount.priv_key, proposalId]);

    // cast vote
    const contract = new VotingContract(
      CONTRACT_ADDRESS,
      CHAIN,
      options.rpcUrl,
      options.privateKey
    );
    const tx = await contract.castVote(
      "0x" +
        Array.from(proof, (byte) => byte.toString(16).padStart(2, "0")).join(
          ""
        ),
      proposalId,
      vote,
      nullifierHash
    );
    console.log("Vote casted successfully at", tx);
  });

program.parse(process.argv);

#!/usr/bin/env tsx

import fs from "fs";
import { Command } from "commander";
import { MerkleTree } from "./lib/merkle-tree";
import { VotingContract } from "./lib/contract";
import { UltraHonkBackend } from "@aztec/bb.js";
import { Noir } from "@noir-lang/noir_js";
import { ZkAccount } from "./lib/account";
import { poseidon2Hash } from "@zkpassport/poseidon2";
import { toHex } from "viem";
import {
  ACCOUNT_MESSAGE,
  CONTRACT_ADDRESS,
  DEFAULT_RPC_URL,
  DEPTH,
  ZERO_LEAF,
} from "./config/constants";

const program = new Command();

program.name("cli").description("Zero Knowledge Voting").version("0.1.0");

program
  .command("account")
  .option("-p, --private-key <private-key>", "Private key")
  .description("Build a zk account")
  .action(async (options) => {
    const zkAccount = await ZkAccount.buildFromPrivateKey(
      options.privateKey,
      ACCOUNT_MESSAGE
    );

    console.log("Commitment:", "0x" + zkAccount.commitment.toString(16));
    console.log("Private key:", "0x" + zkAccount.priv_key.toString(16));
    console.log("Secret:", "0x" + zkAccount.secret.toString(16));
  });

program
  .command("propose")
  .requiredOption(
    "--voters <voters>",
    "File containing the voters",
    "voters.json"
  )
  .requiredOption("--description <description>", "Description of the proposal")
  .requiredOption(
    "--deadline <deadline>",
    "When the proposal will be closed. ISO 8601 timestamp like 2025-07-28T12:00:005Z, you can help yourself with https://www.timestamp-converter.com/"
  )
  .requiredOption("--private-key <private-key>", "Private key")
  .option("--rpc-url <rpc-url>", "RPC URL")
  .description("Propose a new proposal for voting")
  .action(async (options) => {
    const voters = JSON.parse(fs.readFileSync(options.voters, "utf8"));
    const votersTree = new MerkleTree(DEPTH, ZERO_LEAF);
    for (const voter of voters) {
      await votersTree.addCommitment(BigInt(voter));
      console.log(`Added commitment for voter ${voter}`);
    }
    const root = await votersTree.getRoot();

    const deadline = new Date(options.deadline);
    const now = new Date();
    if (deadline < now) {
      throw new Error("Deadline must be in the future");
    }

    console.log(`Creating proposal with root: 0x${root.toString(16)}`);
    const contract = new VotingContract(
      CONTRACT_ADDRESS,
      options.rpcUrl,
      options.privateKey
    );
    const tx = await contract.propose(
      options.description,
      BigInt(Math.floor(deadline.getTime() / 1000)),
      root
    );
    console.log(`Proposal created with id ${tx.id} at ${tx.tx}`);
  });

program
  .command("get-proposal <proposal-id>")
  .option("-r, --rpc-url <rpc-url>", "RPC URL", DEFAULT_RPC_URL)
  .description("Get a proposal")
  .action(async (proposalId, options) => {
    const contract = new VotingContract(CONTRACT_ADDRESS, options.rpcUrl);
    const proposal = await contract.getProposal(proposalId);

    console.log(`Proposal ${proposalId} details\n`);
    console.log(`Description: ${proposal.description}`);
    console.log(
      `Deadline: ${new Date(Number(proposal.deadline) * 1000).toISOString()}`
    );
    console.log(`Root: ${toHex(proposal.votersRoot)}`);
    console.log(`For Votes: ${proposal.forVotes}`);
    console.log(`Against Votes: ${proposal.againstVotes}`);
  });

program
  .command("cast-vote")
  .option("--rpc-url <rpc-url>", "RPC URL", DEFAULT_RPC_URL)
  .option("--voters <voters>", "File containing the voters", "voters.json")
  .requiredOption("--proposal-id <proposal-id>", "Proposal ID")
  .requiredOption("--vote <vote>", "Vote 1 in favor, 0 against")
  .requiredOption("--private-key <private-key>", "Private key")
  .option(
    "--relayer-private-key <relayer-private-key>",
    "Relayer private key. By default it will use the private key of the voter"
  )
  .description("Cast a vote for a proposal")
  .action(async (options) => {
    console.log(
      `Casting vote for proposal ${options.proposalId} with vote ${options.vote} from ${options.rpcUrl}`
    );
    const voters = JSON.parse(fs.readFileSync(options.voters, "utf8"));

    // derive secret and private key for user
    const zkAccount = await ZkAccount.buildFromPrivateKey(
      options.privateKey,
      ACCOUNT_MESSAGE
    );

    // rebuild the tree
    const tree = new MerkleTree(DEPTH, ZERO_LEAF);
    for (const voter of voters) {
      await tree.addCommitment(BigInt(voter));
    }
    const root = await tree.getRoot();
    const { path, directionSelector } = await tree.getProof(
      BigInt(zkAccount.commitment)
    );

    // build proof
    const circuit = JSON.parse(
      fs.readFileSync(
        __dirname + "/../circuits/contracts/assets/bytecode.json",
        "utf8"
      )
    );
    const noir = new Noir(circuit);
    const backend = new UltraHonkBackend(circuit.bytecode);

    // build proof
    const nullifierHash = poseidon2Hash([
      root,
      zkAccount.priv_key,
      BigInt(options.proposalId),
    ]);
    const { witness } = await noir.execute({
      root: `0x${root.toString(16)}`,
      path: path.map((p) => `0x${p.toString(16)}`),
      direction_selector: directionSelector,
      secret: `0x${zkAccount.secret.toString(16)}`,
      priv_key: `0x${zkAccount.priv_key.toString(16)}`,
      nullifier: `0x${nullifierHash.toString(16)}`,
      proposal_id: `0x${options.proposalId.toString(16)}`,
      vote: options.vote,
    });
    const { proof } = await backend.generateProof(witness, {
      keccak: true,
    });

    // cast vote
    const contract = new VotingContract(
      CONTRACT_ADDRESS,
      options.rpcUrl,
      options.relayerPrivateKey ?? options.privateKey
    );
    const tx = await contract.castVote(
      `0x${Array.from(proof, (byte) => byte.toString(16).padStart(2, "0")).join(
        ""
      )}`,
      options.proposalId,
      options.vote,
      nullifierHash
    );
    console.log("Vote casted successfully at", tx);
  });

program.parse(process.argv);

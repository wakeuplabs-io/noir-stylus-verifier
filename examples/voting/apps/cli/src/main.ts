#!/usr/bin/env tsx

import "dotenv/config";
import fs from "fs";
import { Command } from "commander";
import {
  MerkleTree,
  VotingContract,
  ZkAccount,
  Ipfs,
  ProposalMetadata,
  ACCOUNT_MESSAGE,
  DEPTH,
  ZERO_LEAF,
  VotingCircuit,
} from "@voting/core";
import { toHex } from "viem";
import * as prompts from "@clack/prompts";
import color from "picocolors";
import { z } from "zod";

// load env variables from .env file
export const DEFAULT_RPC_URL = process.env.RPC_URL || "";
export const DEFAULT_PRIVATE_KEY = process.env.PRIVATE_KEY as `0x${string}`;
export const DEFAULT_RELAYER_PRIVATE_KEY = process.env
  .RELAYER_PRIVATE_KEY as `0x${string}`;
export const IPFS_PINATA_JWT = process.env.IPFS_PINATA_JWT || "";
export const IPFS_GATEWAY_URL = process.env.IPFS_GATEWAY_URL || "";
export const CONTRACT_ADDRESS = process.env.CONTRACT_ADDRESS as `0x${string}`;

const program = new Command();

const ipfs = new Ipfs(IPFS_PINATA_JWT, IPFS_GATEWAY_URL);

program.name("cli").description("Zero Knowledge Voting").version("0.1.0");

program
  .command("account")
  .option("-p, --private-key <private-key>", "Private key", DEFAULT_PRIVATE_KEY)
  .description("Create a zk account from your evm private key")
  .action(async (options) => {
    prompts.intro(color.inverse("ZK Voting by Wakeup Labs ◌○●"));

    const spinner = prompts.spinner();
    spinner.start("Building zk account...");

    if (!options.privateKey) {
      options.privateKey = toHex(crypto.getRandomValues(new Uint8Array(32)));
    }

    const zkAccount = await ZkAccount.buildFromPrivateKey(
      options.privateKey,
      ACCOUNT_MESSAGE
    );

    spinner.stop("Zk account built successfully");

    prompts.outro(`Your zk account:

    EVM Private key: ${options.privateKey}
    ZK Address: 0x${zkAccount.address.toString(16)}
    ZK Private key: 0x${zkAccount.privateKey.toString(16)}
    ZK Secret: 0x${zkAccount.secret.toString(16)}
    `);
  });

program
  .command("propose")
  .requiredOption(
    "--proposal <path to proposal>",
    "File containing the proposal",
    "proposal.json"
  )
  .option("--private-key <private-key>", "Private key", DEFAULT_PRIVATE_KEY)
  .option("--rpc-url <rpc-url>", "RPC URL", DEFAULT_RPC_URL)
  .description("Make a proposal for voting")
  .action(async (options) => {
    prompts.intro(color.inverse("ZK Voting by Wakeup Labs ◌○●"));

    if (!options.privateKey) {
      prompts.cancel("Private key is required");
      return;
    }

    const proposalJSON = JSON.parse(fs.readFileSync(options.proposal, "utf8"));
    const proposal = {
      ...proposalJSON,
      deadline: new Date(proposalJSON.deadline),
    };
    const proposalSchema = z.object({
      title: z.string(),
      body: z.string(),
      deadline: z.date(),
      voters: z.array(z.string()),
    });
    const parsedProposal = proposalSchema.safeParse(proposal);
    if (parsedProposal.error) {
      prompts.cancel(`Invalid proposal file: ${parsedProposal.error.message}`);
      return;
    }

    if (proposal.deadline < new Date()) {
      prompts.cancel("Deadline must be in the future");
      return;
    }

    const spinner = prompts.spinner();
    spinner.start("Building voters tree...");

    const votersTree = new MerkleTree(DEPTH, ZERO_LEAF);
    for (const voter of proposal.voters) {
      await votersTree.addCommitment(BigInt(voter));
    }
    const root = await votersTree.getRoot();

    spinner.stop(
      `Voters tree built successfully with root: 0x${root.toString(16)}`
    );

    spinner.start("Uploading metadata to IPFS...");

    const metadata = await ipfs.uploadJSON(proposal);
    
    spinner.stop(`Metadata uploaded to IPFS: ${metadata}`);

    spinner.start("Creating proposal...");

    const contract = new VotingContract(
      CONTRACT_ADDRESS,
      options.rpcUrl,
      options.privateKey
    );
    const tx = await contract.propose(
      metadata,
      BigInt(Math.floor(proposal.deadline.getTime() / 1000)),
      root
    );
    spinner.stop(`Proposal created in tx ${tx.tx}`);

    prompts.outro(`Your proposal is ready:

    Proposal ID: ${tx.id}
    Proposal Metadata CID: ${metadata}
    `);
  });

program
  .command("get-proposal <proposal-id>")
  .option("--rpc-url <rpc-url>", "RPC URL", DEFAULT_RPC_URL)
  .description("Get a proposal")
  .action(async (proposalId, options) => {
    prompts.intro(color.inverse("ZK Voting by Wakeup Labs ◌○●"));

    const spinner = prompts.spinner();
    spinner.start("Getting proposal from the blockchain...");

    const contract = new VotingContract(CONTRACT_ADDRESS, options.rpcUrl);
    const proposal = await contract.getProposal(proposalId);

    spinner.stop(`Got proposal ${proposalId} votes`);

    prompts.outro(`Proposal ${proposalId} votes\n

    For Votes: ${proposal.forVotes}
    Against Votes: ${proposal.againstVotes}
    `);
  });

program
  .command("cast-vote")
  .option("--rpc-url <rpc-url>", "RPC URL", DEFAULT_RPC_URL)
  .option("--private-key <private-key>", "Private key", DEFAULT_PRIVATE_KEY)
  .requiredOption("--proposal-id <proposal-id>", "Proposal ID")
  .requiredOption("--vote <vote>", "Vote 1 in favor, 0 against")
  .option(
    "--relayer-private-key <relayer-private-key>",
    "Relayer private key. By default it will use the private key of the voter"
  )
  .description("Cast a vote for a proposal")
  .action(async (options) => {
    prompts.intro(color.inverse("ZK Voting by Wakeup Labs ◌○●"));

    if (!options.privateKey) {
      prompts.cancel("Private key is required");
      return;
    }

    const spinner = prompts.spinner();
    spinner.start("Getting proposal from the blockchain...");

    const contract = new VotingContract(
      CONTRACT_ADDRESS,
      options.rpcUrl,
      options.relayerPrivateKey ?? options.privateKey
    );

    const metadata = await contract.getProposalMetadata(options.proposalId);
    const proposal = (await ipfs.downloadJSON(metadata)) as ProposalMetadata;

    spinner.stop(`Got proposal ${options.proposalId} details`);

    spinner.start("Recovering zk account...");

    // derive secret and private key for user
    const zkAccount = await ZkAccount.buildFromPrivateKey(
      options.privateKey,
      ACCOUNT_MESSAGE
    );

    // check account is in the voters tree
    const isVoter = proposal.voters.find(
      (voter: string) => BigInt(voter) === zkAccount.address
    );
    if (!isVoter) {
      prompts.cancel("You are not a voter for this proposal");
      return;
    }

    spinner.stop("Zk account recovered");

    spinner.start("Building proof...");

    const tree = new MerkleTree(DEPTH, ZERO_LEAF);
    for (const voter of proposal.voters) {
      await tree.addCommitment(BigInt(voter));
    }
    const root = await tree.getRoot();
    const { path, directionSelector } = await tree.getProof(
      BigInt(zkAccount.address)
    );

    const nullifier = await VotingCircuit.generateNullifier(
      root,
      zkAccount.privateKey,
      BigInt(options.proposalId)
    );

    const proof = await VotingCircuit.generateProof(
      options.proposalId,
      options.vote,
      root,
      path,
      directionSelector,
      zkAccount.privateKey,
      zkAccount.secret,
      nullifier
    );

    spinner.stop("Proof built successfully");

    spinner.start("Casting vote...");

    const tx = await contract.castVote(
      `0x${Array.from(proof, (byte) => byte.toString(16).padStart(2, "0")).join(
        ""
      )}`,
      options.proposalId,
      options.vote,
      nullifier
    );

    spinner.stop(`Vote casted successfully at ${tx}`);

    prompts.outro(`You're all set!`);
  });

program.parse(process.argv);

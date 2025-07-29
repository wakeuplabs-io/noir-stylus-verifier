#!/usr/bin/env tsx

import "dotenv/config";
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
  DEPTH,
  DEFAULT_PRIVATE_KEY,
  ZERO_LEAF,
  DEFAULT_RPC_URL,
} from "./config/constants";
import * as prompts from "@clack/prompts";
import color from "picocolors";
import { Ipfs } from "./lib/ipfs";
import { ProposalMetadata } from "./types/proposal";
import { z } from "zod";

const program = new Command();

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
    ZK Address: 0x${zkAccount.commitment.toString(16)}
    ZK Private key: 0x${zkAccount.priv_key.toString(16)}
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
    const metadata = await Ipfs.uploadJSON(proposal);
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
    const proposal = (await Ipfs.downloadJSON(metadata)) as ProposalMetadata;

    spinner.stop(`Got proposal ${options.proposalId} details`);

    spinner.start("Recovering zk account...");

    // derive secret and private key for user
    const zkAccount = await ZkAccount.buildFromPrivateKey(
      options.privateKey,
      ACCOUNT_MESSAGE
    );

    // check account is in the voters tree
    const isVoter = proposal.voters.find(
      (voter: string) => BigInt(voter) === zkAccount.commitment
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
      BigInt(zkAccount.commitment)
    );

    const circuit = JSON.parse(
      fs.readFileSync(
        __dirname + "/../../../circuits/contracts/assets/bytecode.json",
        "utf8"
      )
    );
    const noir = new Noir(circuit);
    const backend = new UltraHonkBackend(circuit.bytecode);

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

    spinner.stop("Proof built successfully");

    spinner.start("Casting vote...");

    // cast vote
    const tx = await contract.castVote(
      `0x${Array.from(proof, (byte) => byte.toString(16).padStart(2, "0")).join(
        ""
      )}`,
      options.proposalId,
      options.vote,
      nullifierHash
    );
    spinner.stop(`Vote casted successfully at ${tx}`);

    prompts.outro(`You're all set!`);
  });

program.parse(process.argv);

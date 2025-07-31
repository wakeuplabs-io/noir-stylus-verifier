#!/usr/bin/env tsx

import "dotenv/config";
import fs from "fs";
import { Command } from "commander";
import {
  MerkleTree,
  VotingContract,
  ZkAccount,
  ACCOUNT_MESSAGE,
  DEPTH,
  ZERO_LEAF,
  VotingCircuit,
  PinataIpfs,
} from "@voting/core";
import { createWalletClient, http } from "viem";
import { privateKeyToAccount, privateKeyToAddress } from "viem/accounts";
import * as prompts from "@clack/prompts";
import color from "picocolors";
import {
  RPC_URL,
  PRIVATE_KEY,
  IPFS_PINATA_JWT,
  IPFS_GATEWAY_URL,
  CONTRACT_ADDRESS,
  CHAIN_ID,
} from "./env";
import z from "zod";

const contract = new VotingContract(
  new PinataIpfs(IPFS_PINATA_JWT, IPFS_GATEWAY_URL),
  CHAIN_ID,
  CONTRACT_ADDRESS,
  RPC_URL
);

export const walletClient = createWalletClient({
  transport: http(RPC_URL),
});

const program = new Command();

program
  .name("cli")
  .description("Zero Knowledge Voting with Noir and Stylus")
  .version("0.1.0");

program
  .command("account")
  .description("Create a zk account from your evm private key")
  .action(async () => {
    prompts.intro(color.inverse("ZK Voting by Wakeup Labs ◌○●"));

    const spinner = prompts.spinner();
    spinner.start("Building zk account...");

    const zkAccount = await ZkAccount.buildFromPrivateKey(
      PRIVATE_KEY,
      ACCOUNT_MESSAGE
    );

    spinner.stop("Zk account built successfully");

    prompts.outro(`Your zk account:

    EVM Private key: ${PRIVATE_KEY}
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
  .description("Make a proposal for voting")
  .action(async (options) => {
    prompts.intro(color.inverse("ZK Voting by Wakeup Labs ◌○●"));

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

    spinner.start("Creating proposal...");

    const txRequest = await contract.preparePropose(
      privateKeyToAddress(PRIVATE_KEY),
      proposal,
      BigInt(Math.floor(proposal.deadline.getTime() / 1000)),
      root
    );
    const tx = await walletClient.sendTransaction({
      ...txRequest,
      account: privateKeyToAccount(PRIVATE_KEY),
    });

    spinner.stop(`Proposal created in tx ${tx}`);

    spinner.start("Recovering proposal id...");

    const proposalId = await contract.recoverProposalId(tx);

    spinner.stop(`Proposal id recovered`);

    prompts.outro(`Proposal created with id: ${proposalId}`);
  });

program
  .command("get-proposal <proposal-id>")
  .description("Get a proposal")
  .action(async (proposalId) => {
    prompts.intro(color.inverse("ZK Voting by Wakeup Labs ◌○●"));

    const spinner = prompts.spinner();
    spinner.start("Getting proposal from the blockchain...");

    const proposal = await contract.getProposal(proposalId);

    spinner.stop(`Got details for proposal with id ${proposalId}`);

    prompts.outro(`Proposal ${proposalId} votes:\n

    For Votes: ${proposal.for}
    Against Votes: ${proposal.against}
    `);
  });

program
  .command("cast-vote")
  .requiredOption("--proposal-id <proposal-id>", "Proposal ID")
  .requiredOption("--vote <vote>", "Vote 1 in favor, 0 against")
  .description("Cast a vote for a proposal")
  .action(async (options) => {
    prompts.intro(color.inverse("ZK Voting by Wakeup Labs ◌○●"));

    const spinner = prompts.spinner();
    spinner.start("Getting proposal from the blockchain...");

    const proposal = await contract.getProposal(options.proposalId);

    spinner.stop(`Got details for proposal with id ${options.proposalId}`);

    spinner.start("Recovering zk account...");

    // derive secret and private key for user
    const zkAccount = await ZkAccount.buildFromPrivateKey(
      PRIVATE_KEY,
      ACCOUNT_MESSAGE
    );

    // check account is in the voters tree
    const isVoter = proposal.metadata.voters.find(
      (voter: string) => BigInt(voter) === zkAccount.address
    );
    if (!isVoter) {
      prompts.cancel("You are not a voter for this proposal");
      return;
    }

    spinner.stop("Zk account recovered");

    spinner.start("Building proof...");

    const tree = new MerkleTree(
      DEPTH,
      ZERO_LEAF,
      proposal.metadata.voters.map((voter) => BigInt(voter))
    );
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

    const txRequest = await contract.prepareCastVote(
      privateKeyToAddress(PRIVATE_KEY),
      `0x${Array.from(proof, (byte) => byte.toString(16).padStart(2, "0")).join(
        ""
      )}`,
      options.proposalId,
      options.vote,
      nullifier
    );
    const tx = await walletClient.sendTransaction({
      ...txRequest,
      account: privateKeyToAccount(PRIVATE_KEY),
    });

    spinner.stop(`Vote casted successfully at ${tx}`);

    prompts.outro(`You're all set!`);
  });

program.parse(process.argv);

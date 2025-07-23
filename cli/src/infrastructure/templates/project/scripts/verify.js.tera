#!/usr/bin/env node

import fs from "fs";
import { UltraHonkBackend } from "@aztec/bb.js";
import { Noir } from "@noir-lang/noir_js";
import { createPublicClient, http, parseAbi } from "viem";

const VERIFIER_ADDRESS = process.env.VERIFIER_ADDRESS;
const RPC_ADDRESS = process.env.RPC_ADDRESS;

if (!VERIFIER_ADDRESS || !RPC_ADDRESS) {
  console.error("VERIFIER_ADDRESS and RPC_ADDRESS must be set");
  process.exit(1);
}

const client = createPublicClient({
  transport: http(RPC_ADDRESS),
});

try {
  const vk = fs.readFileSync("./contracts/assets/vk", "hex");
  const circuit = JSON.parse(fs.readFileSync("./contracts/assets/bytecode.json", "utf8"));

  const noir = new Noir(circuit);
  const backend = new UltraHonkBackend(circuit.bytecode);

  console.log("Executing circuit...");
  const { witness } = await noir.execute({ x: 1, y: 2, z: 3 });

  console.log("Generating proof...");
  const { proof, publicInputs } = await backend.generateProof(witness, {
    keccak: true,
  });
  console.log("Public inputs:", publicInputs);

  console.log("Verifying proof with contract...");
  const result = await client.readContract({
    address: VERIFIER_ADDRESS,
    functionName: "verify",
    abi: parseAbi([
      "function verify(bytes proof, bytes y, bytes z) view returns (bool)",
    ]),
    args: [
      "0x" + Array.from(proof, (byte) => byte.toString(16).padStart(2, "0")).join(""),
      publicInputs[0],
      publicInputs[1],
    ],
  });

  console.log("Result:", result);

  process.exit(0);
} catch (error) {
  console.error(error);
  process.exit(1);
}

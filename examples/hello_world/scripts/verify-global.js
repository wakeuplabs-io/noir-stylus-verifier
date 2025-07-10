import fs from "fs";
import { UltraHonkBackend } from "@aztec/bb.js";
import { Noir } from "@noir-lang/noir_js";
import { createPublicClient, http } from "viem";

const GLOBAL_VERIFIER_ADDRESS = "0x2f9f4741ab606632718f7bda0bf5c79e1dd03ac3";
const RPC_ADDRESS = "http://127.0.0.1:8547";

const client = createPublicClient({
  transport: http(RPC_ADDRESS),
});

function encodeProof(proof) {
  return (
    "0x" +
    Array.from(proof, (byte) => byte.toString(16).padStart(2, "0")).join("")
  );
}

function encodePublicInputs(publicInputs) {
  return "0x" + publicInputs.map((i) => i.slice(2)).join("");
}

function encodeVk(vk) {
  return "0x" + vk;
}

try {
  const vk = fs.readFileSync("./circuit/target/vk", "hex");
  const circuit = JSON.parse(
    fs.readFileSync("./circuit/target/hello_world.json", "utf8")
  );

  const noir = new Noir(circuit);
  const backend = new UltraHonkBackend(circuit.bytecode);

  console.log("Executing circuit...");
  const { witness } = await noir.execute({ x: 1, y: 2 });

  console.log("Generating proof...");
  const { proof, publicInputs } = await backend.generateProof(witness, {
    keccak: true,
  });

  console.log("Verifying proof with contract...");
  const result = await client.readContract({
    address: GLOBAL_VERIFIER_ADDRESS,
    abi: [
      {
        inputs: [
          {
            internalType: "bytes",
            name: "proof",
            type: "bytes",
          },
          {
            internalType: "bytes",
            name: "public_inputs",
            type: "bytes",
          },
          {
            internalType: "bytes",
            name: "vk",
            type: "bytes",
          },
        ],
        outputs: [
          {
            internalType: "bool",
            name: "result",
            type: "bool",
          },
        ],
        stateMutability: "view",
        type: "function",
        name: "verify",
      },
    ],
    functionName: "verify",
    args: [encodeProof(proof), encodePublicInputs(publicInputs), encodeVk(vk)],
  });

  console.log("Result:", result);

  process.exit(0);
} catch (error) {
  console.error(error);
  process.exit(1);
}

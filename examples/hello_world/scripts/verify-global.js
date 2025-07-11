import fs from "fs";
import { UltraHonkBackend } from "@aztec/bb.js";
import { Noir } from "@noir-lang/noir_js";
import { createPublicClient, http, parseAbi } from "viem";

const GLOBAL_VERIFIER_ADDRESS = "0x2f9f4741ab606632718f7bda0bf5c79e1dd03ac3";
const RPC_ADDRESS = "http://127.0.0.1:8547";

const client = createPublicClient({
  transport: http(RPC_ADDRESS),
});

try {
  const vk = "0x" + fs.readFileSync("./circuit/target/vk", "hex");
  const circuit = JSON.parse(
    fs.readFileSync("./circuit/target/hello_world.json", "utf8")
  );

  const noir = new Noir(circuit);
  const backend = new UltraHonkBackend(circuit.bytecode);

  console.log("Executing circuit...");
  const { witness } = await noir.execute({ x: 1, y: 2, z: 3 });

  console.log("Generating proof...");
  const { proof, publicInputs } = await backend.generateProof(witness, {
    keccak: true,
  });

  console.log("Verifying proof with contract...");
  const result = await client.readContract({
    functionName: "verify",
    address: GLOBAL_VERIFIER_ADDRESS,
    abi: parseAbi([
      "function verify(bytes proof, bytes public_inputs, bytes vk) view returns (bool)",
    ]),
    args: [
      "0x" + Array.from(proof, (byte) => byte.toString(16).padStart(2, "0")).join(""),
      "0x" + publicInputs.map((i) => i.slice(2)).join(""),
      vk,
    ],
  });

  console.log("Result:", result);

  process.exit(0);
} catch (error) {
  console.error(error);
  process.exit(1);
}

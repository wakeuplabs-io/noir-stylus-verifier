import fs from "fs";
import { UltraHonkBackend } from "@aztec/bb.js";
import { Noir } from "@noir-lang/noir_js";
import { createPublicClient, http, parseAbi } from "viem";

const VERIFIER_ADDRESS = "0x79693edb49473dc3522de16fbd047977c4999d5c";
const RPC_ADDRESS = "http://127.0.0.1:8547";

const client = createPublicClient({
  transport: http(RPC_ADDRESS),
});

const encodeProof = (proof) =>
  "0x" +
  Array.from(proof, (byte) => byte.toString(16).padStart(2, "0")).join("");

const encodePublicInputs = (publicInputs) =>
  "0x" + publicInputs.map((i) => i.slice(2)).join("");

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
    address: VERIFIER_ADDRESS,
    abi: parseAbi([
      "function verify(bytes proof, bytes public_inputs) view returns (bool)",
    ]),
    functionName: "verify",
    args: [encodeProof(proof), encodePublicInputs(publicInputs)],
  });

  console.log("Result:", result);

  process.exit(0);
} catch (error) {
  console.error(error);
  process.exit(1);
}

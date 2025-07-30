import { UltraHonkBackend } from "@aztec/bb.js";
import { Noir } from "@noir-lang/noir_js";
import { poseidon2Hash } from "@zkpassport/poseidon2";
import circuit from "../config/bytecode.json";

export class VotingCircuit {
  static async generateProof(
    proposal_id: bigint,
    vote: boolean,
    root: bigint,
    path: bigint[],
    direction_selector: boolean[],
    priv_key: bigint,
    secret: bigint,
    nullifier: bigint
  ) {
    console.log("ABS 1")
    const noir = new Noir(circuit as any);
    console.log("ABS 2")
    const backend = new UltraHonkBackend(circuit.bytecode);
    console.log("ABS 3")
    const { witness } = await noir.execute({
      root: `0x${root.toString(16)}`,
      path: path.map((p) => `0x${p.toString(16)}`),
      direction_selector: direction_selector,
      secret: `0x${secret.toString(16)}`,
      priv_key: `0x${priv_key.toString(16)}`,
      nullifier: `0x${nullifier.toString(16)}`,
      proposal_id: `0x${proposal_id.toString(16)}`,
      vote: vote ? 1 : 0,
    });
    console.log("ABS 4")
    const { proof } = await backend.generateProof(witness, {
      keccak: true,
    });
    console.log("ABS 5")
    return proof;
  }

  static async generateNullifier(
    root: bigint,
    priv_key: bigint,
    proposal_id: bigint
  ) {
    return poseidon2Hash([root, priv_key, proposal_id]);
  }
}

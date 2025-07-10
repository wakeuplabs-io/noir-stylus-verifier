import { UltraHonkBackend } from '@aztec/bb.js';
import { Noir } from '@noir-lang/noir_js';
import fs from 'fs';

try {
    const circuit = JSON.parse(fs.readFileSync("./circuit/target/hello_world.json", "utf8"));
    const noir = new Noir(circuit);
    const backend = new UltraHonkBackend(circuit.bytecode);

    console.log("Executing circuit...");
    const { witness } = await noir.execute({ x: 1, y: 2 });

    console.log("Generating proof...");
    const proof = await backend.generateProof(witness, { keccak: true });
    const computedProof = Array.from(proof.proof, byte => byte.toString(16).padStart(2, '0')).join('')
    const computedPublicInputs = proof.publicInputs.map(i => i.slice(2)).join('')

    console.log("proof", computedProof);
    console.log("public_inputs", computedPublicInputs);

    process.exit(0);
} catch (error) {
    console.error(error);
    process.exit(1);
}
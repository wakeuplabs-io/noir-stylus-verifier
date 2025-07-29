import { keccak256 } from "viem";
import { poseidon2Hash } from "@zkpassport/poseidon2";
import { privateKeyToAccount } from "viem/accounts";

export class ZkAccount {
  static async buildFromSignature(signature: `0x${string}`) {
    const hash = keccak256(signature);

    const seed = BigInt(hash);
    const priv_key = poseidon2Hash([seed, 0n]);
    const secret = poseidon2Hash([seed, 1n]);
    const commitment = poseidon2Hash([priv_key, secret]);

    return {
      commitment,
      priv_key,
      secret,
    };
  }

  static async buildFromPrivateKey(privateKey: `0x${string}`, message: string) {
    const account = privateKeyToAccount(privateKey.startsWith("0x") ? privateKey : `0x${privateKey}`);
    const signedMessage = await account.signMessage({ message });
    return this.buildFromSignature(signedMessage);
  }
}

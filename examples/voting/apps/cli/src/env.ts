import "dotenv/config";
import z from "zod";

const hexString = (length: number) => new RegExp(`^0x[0-9a-fA-F]{${length}}$`);

const envSchema = z.object({
  RPC_URL: z.string(),
  PRIVATE_KEY: z.string().regex(hexString(64), {
    message: "PRIVATE_KEY must be a 0x-prefixed 64-character hex string",
  }),
  RELAYER_PRIVATE_KEY: z
    .string()
    .regex(hexString(64), {
      message:
        "RELAYER_PRIVATE_KEY must be a 0x-prefixed 64-character hex string",
    })
    .optional(), // Optional: fallback to voter key at runtime
  IPFS_PINATA_JWT: z.string(),
  IPFS_GATEWAY_URL: z.string(),
  CONTRACT_ADDRESS: z.string().regex(/^0x[0-9a-fA-F]{40}$/, {
    message:
      "CONTRACT_ADDRESS must be a valid 0x-prefixed 40-character hex address",
  }),
  CHAIN_ID: z.coerce
    .number()
    .int()
    .positive({ message: "CHAIN_ID must be a positive integer" }),
});

const env = envSchema.parse(process.env);

export const RPC_URL = env.RPC_URL;
export const PRIVATE_KEY = env.PRIVATE_KEY as `0x${string}`;
export const RELAYER_PRIVATE_KEY = env.RELAYER_PRIVATE_KEY as `0x${string}`;
export const IPFS_PINATA_JWT = env.IPFS_PINATA_JWT;
export const IPFS_GATEWAY_URL = env.IPFS_GATEWAY_URL;
export const CONTRACT_ADDRESS = env.CONTRACT_ADDRESS as `0x${string}`;
export const CHAIN_ID = env.CHAIN_ID;

import "dotenv/config";
import z from "zod";

const envSchema = z.object({
  RPC_URL: z.string(),
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
export const CONTRACT_ADDRESS = env.CONTRACT_ADDRESS as `0x${string}`;
export const CHAIN_ID = env.CHAIN_ID;

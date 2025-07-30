// Depth of the merkle tree
export const DEPTH = 2;

// Zero leaf for the merkle tree
export const ZERO_LEAF = BigInt(0);

// Message to sign to create a zk account
export const ACCOUNT_MESSAGE = "Access your voting account";

// Supported chain ids
export enum SupportedChainId {
    ARBITRUM_SEPOLIA = 421614,
}

// Contract addresses
export const CONTRACT_ADDRESS: Record<SupportedChainId, `0x${string}`> = {
    [SupportedChainId.ARBITRUM_SEPOLIA]: "0xa3f2a1a0a0cb4bd272867f4b7fc5e8d634ae7533",
}

// Default RPC URLs
export const DEFAULT_RPC_URL: Record<SupportedChainId, string> = {
    [SupportedChainId.ARBITRUM_SEPOLIA]: "https://sepolia-rollup.arbitrum.io/rpc",
}

// Multicall address
export const MULTICALL_ADDRESS: Record<SupportedChainId, `0x${string}`> = {
    [SupportedChainId.ARBITRUM_SEPOLIA]: "0xca11bde05977b3631167028862be2a173976ca11",
}
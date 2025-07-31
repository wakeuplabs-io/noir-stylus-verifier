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

// Multicall address
export const MULTICALL_ADDRESS: Record<SupportedChainId, `0x${string}`> = {
    [SupportedChainId.ARBITRUM_SEPOLIA]: "0xca11bde05977b3631167028862be2a173976ca11",
}
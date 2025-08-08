export const NUMBER_OF_SHIPS = 5;

export const BOARD_SIZE = 10;

// Supported chain ids
export enum SupportedChainId {
    ARBITRUM_SEPOLIA = 421614,
}

// Multicall address
export const MULTICALL_ADDRESS: Record<SupportedChainId, `0x${string}`> = {
    [SupportedChainId.ARBITRUM_SEPOLIA]: "0xca11bde05977b3631167028862be2a173976ca11",
}
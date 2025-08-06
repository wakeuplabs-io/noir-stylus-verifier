// Depth of the merkle tree
export const DEPTH = 2;

export const NUMBER_OF_SHIPS = 5;

export const SHIP_LENGTHS = [
    5, // Carrier
    4, // Battleship
    3, // Cruiser
    3, // Submarine
    2, // Destroyer
];

// Supported chain ids
export enum SupportedChainId {
    ARBITRUM_SEPOLIA = 421614,
}

// Multicall address
export const MULTICALL_ADDRESS: Record<SupportedChainId, `0x${string}`> = {
    [SupportedChainId.ARBITRUM_SEPOLIA]: "0xca11bde05977b3631167028862be2a173976ca11",
}
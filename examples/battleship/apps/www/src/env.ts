const CONTRACT_ADDRESS = import.meta.env.VITE_CONTRACT_ADDRESS;
const CHAIN_ID = Number(import.meta.env.VITE_CHAIN_ID);
const RPC_URL = import.meta.env.VITE_RPC_URL;

if (!CONTRACT_ADDRESS || !RPC_URL || !CHAIN_ID) {
  throw new Error("CONTRACT_ADDRESS, RPC_URL and CHAIN_ID must be set");
}

export { CONTRACT_ADDRESS, RPC_URL, CHAIN_ID };

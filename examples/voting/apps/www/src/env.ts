const IPFS_PINATA_JWT = import.meta.env.VITE_IPFS_PINATA_JWT;
const IPFS_GATEWAY_URL = import.meta.env.VITE_IPFS_GATEWAY_URL;

if (!IPFS_PINATA_JWT || !IPFS_GATEWAY_URL) {
  throw new Error("IPFS_PINATA_JWT and IPFS_GATEWAY_URL must be set");
}

export { IPFS_PINATA_JWT, IPFS_GATEWAY_URL };

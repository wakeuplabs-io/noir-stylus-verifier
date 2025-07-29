import { PinataSDK } from "pinata";
import { IPFS_PINATA_JWT, IPFS_GATEWAY_URL } from "../config/constants";

export class Ipfs {
  private static client: PinataSDK;

  static async uploadJSON(json: Object): Promise<string> {
    const result = await this.getClient().upload.public.json(json);
    return result.cid;
  }

  static async downloadJSON(cid: string): Promise<Object> {
    const result = await this.getClient().gateways.public.get(cid);
    if (result.contentType !== "application/json") {
      throw new Error("Content type is not JSON");
    }

    return result.data as Object;
  }

  private static getClient() {
    if (!this.client) {
      this.client = new PinataSDK({
        pinataJwt: IPFS_PINATA_JWT,
        pinataGateway: IPFS_GATEWAY_URL
      })
    }
    return this.client;
  }
}
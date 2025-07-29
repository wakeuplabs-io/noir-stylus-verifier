import { PinataSDK } from "pinata";

export class Ipfs {
  private client: PinataSDK;

  constructor(pinataJwt: string, pinataGateway: string) {
    this.client = new PinataSDK({
      pinataJwt,
      pinataGateway,
    });
  }

  async uploadJSON(json: Object): Promise<string> {
    const result = await this.client.upload.public.json(json);
    return result.cid;
  }

  async downloadJSON(cid: string): Promise<Object> {
    const result = await this.client.gateways.public.get(cid);
    if (result.contentType !== "application/json") {
      throw new Error("Content type is not JSON");
    }

    return result.data as Object;
  }
}

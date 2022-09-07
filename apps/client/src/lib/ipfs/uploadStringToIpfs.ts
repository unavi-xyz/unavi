import { ipfsClient } from "./client";

export async function uploadStringToIpfs(str: string) {
  try {
    const buffer = new TextEncoder().encode(str);
    const { cid } = await ipfsClient.add(buffer);
    const hash = cid.toString();

    return `ipfs://${hash}`;
  } catch (error) {
    console.error(error);
  }
}

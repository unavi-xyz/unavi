import { ipfsClient } from "./client";

export async function uploadFileToIpfs(file: File) {
  try {
    const buffer = await new Response(file).arrayBuffer();
    const { cid } = await ipfsClient.add(buffer);
    const hash = cid.toString();

    return `ipfs://${hash}`;
  } catch (error) {
    console.error(error);
  }
}

import { ipfsClient } from "./client";

export async function loadFromIpfs(hash: string) {
  try {
    const res = await ipfsClient.cat(hash);

    const files: Uint8Array[] = [];
    for await (const file of res) {
      files.push(file);
    }

    const blob = new Blob(files);
    const url = URL.createObjectURL(blob);

    return url;
  } catch (error) {
    console.error(error);
  }
}

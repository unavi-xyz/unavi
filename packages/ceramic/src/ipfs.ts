import { IPFS } from "ipfs-core";

export async function loadFromIpfs(ipfs: IPFS, cid: string) {
  const res = ipfs.cat(cid);

  const files: Uint8Array[] = [];
  for await (const file of res) {
    files.push(file);
  }

  const blob = new Blob(files);
  const url = URL.createObjectURL(blob);
  return url;
}

export async function uploadFileToIpfs(ipfs: IPFS, file: File) {
  const { cid } = await ipfs.add(file);
  return `ipfs://${cid.toString()}`;
}

import { IPFS } from "ipfs-core";

export async function loadImage(ipfs: IPFS, cid: string) {
  const res = ipfs.cat(cid);

  const files: Uint8Array[] = [];
  for await (const file of res) {
    files.push(file);
  }

  const blob = new Blob(files);
  const url = URL.createObjectURL(blob);
  return url;
}

export async function uploadImageToIpfs(image: File) {
  const body = new FormData();
  body.append("path", image, image.name);
  const res = await fetch(`https://ipfs.infura.io:5001/api/v0/add`, {
    method: "POST",
    body,
  });
  const { Hash } = await res.json();
  return `ipfs://${Hash}`;
}

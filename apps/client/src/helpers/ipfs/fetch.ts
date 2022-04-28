import { useIpfsStore } from "./store";

let cache: { [key: string]: string } = {};

export async function loadFromIpfs(hash: string) {
  //check cache
  if (cache[hash]) return cache[hash];

  const ipfs = useIpfsStore.getState().ipfs;

  const res = await ipfs.cat(hash);

  const files: Uint8Array[] = [];
  for await (const file of res) {
    files.push(file);
  }

  const blob = new Blob(files);
  const url = URL.createObjectURL(blob);

  //set cache
  cache[hash] = url;

  return url;
}

export async function getIpfsUrl(path: string) {
  //if path is an ipfs hash, load it
  if (path.startsWith("ipfs://")) {
    const hash = path.replace("ipfs://", "");
    const url = await loadFromIpfs(hash);
    return url;
  }

  //otherwise, assume it's an http url
  return path;
}

export async function uploadFileToIpfs(file: File) {
  const ipfs = useIpfsStore.getState().ipfs;

  const buffer = await new Response(file).arrayBuffer();
  const { cid } = await ipfs.add(buffer);
  const hash = cid.toString();

  //set cache
  cache[hash] = URL.createObjectURL(file);

  return `ipfs://${hash}`;
}

export async function uploadStringToIpfs(str: string) {
  const ipfs = useIpfsStore.getState().ipfs;

  const buffer = new TextEncoder().encode(str);
  const { cid } = await ipfs.add(buffer);
  const hash = cid.toString();

  //set cache
  cache[hash] = str;

  return `ipfs://${hash}`;
}

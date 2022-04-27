import { useIpfsStore } from "./store";

export async function loadFromIpfs(hash: string) {
  const ipfs = useIpfsStore.getState().ipfs;

  const res = await ipfs.cat(hash);

  const files: Uint8Array[] = [];
  for await (const file of res) {
    files.push(file);
  }

  const blob = new Blob(files);
  const url = URL.createObjectURL(blob);
  return url;
}

export async function pathToUrl(path: string) {
  //if path is an ipfs hash, load it
  if (path.startsWith("ipfs://")) {
    const hash = path.replace("ipfs://", "");
    const url = await loadFromIpfs(hash);
    return url;
  }

  //otherwise, assume it's an http url
  return path;
}

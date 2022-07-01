export function getIpfsUrlSSR(path: string) {
  //if path is an ipfs hash, load it
  if (path.startsWith("ipfs://")) {
    const hash = path.replace("ipfs://", "");
    return `https://ipfs.infura.io/ipfs/${hash}`;
  }

  //otherwise, assume it's a http url
  return path;
}

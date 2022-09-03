export function getIpfsUrlSSR(path: string) {
  // If IPFS
  if (path.startsWith("ipfs://")) {
    if (!process.env.NEXT_PUBLIC_IPFS_GATEWAY)
      throw new Error("NEXT_PUBLIC_IPFS_GATEWAY not set");
    const hash = path.replace("ipfs://", "");
    return `https://${process.env.NEXT_PUBLIC_IPFS_GATEWAY}/ipfs/${hash}`;
  }

  // Otherwise, assume it's a http url
  return path;
}

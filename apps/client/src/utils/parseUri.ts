import { env } from "../env/client.mjs";

export function parseUri(uri: string) {
  const isData = uri.startsWith("data:");
  const isIPFS = uri.startsWith("ipfs://");
  const isUrl = uri.startsWith("http://") || uri.startsWith("https://");

  if (isData) {
    return uri;
  } else if (isIPFS) {
    const hash = uri.split("ipfs://")[1];
    const http =
      env.NEXT_PUBLIC_IPFS_GATEWAY.split(":")[0] === "localhost"
        ? "http"
        : "https";
    return `${http}://${env.NEXT_PUBLIC_IPFS_GATEWAY}/ipfs/${hash}`;
  } else if (isUrl) {
    return uri;
  } else {
    // Assume relative path
    return uri;
  }
}

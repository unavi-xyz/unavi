import { create } from "ipfs-http-client";

import { env } from "../../env/client.mjs";

const authKey = env.NEXT_PUBLIC_IPFS_AUTH;

export const ipfsClient = create({
  url: env.NEXT_PUBLIC_IPFS_ENDPOINT,
  headers: authKey
    ? {
        Authorization: `Basic ${Buffer.from(authKey).toString("base64")}`,
      }
    : undefined,
  protocol: process.env.NODE_ENV === "development" ? "http" : "https",
});

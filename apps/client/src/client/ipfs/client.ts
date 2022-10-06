import { create } from "ipfs-http-client";

import { env } from "../../env/client.mjs";

const authKey = env.NEXT_PUBLIC_IPFS_AUTH;
if (!authKey) throw new Error("NEXT_PUBLIC_IPFS_AUTH is not set");

export const ipfsClient = create({
  url: env.NEXT_PUBLIC_IPFS_ENDPOINT,
  headers: {
    Authorization: `Basic ${Buffer.from(authKey).toString("base64")}`,
  },
});

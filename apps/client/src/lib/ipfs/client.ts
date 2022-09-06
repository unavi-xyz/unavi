import { create } from "ipfs-http-client";

const authKey = process.env.NEXT_PUBLIC_IPFS_AUTH;
if (!authKey) throw new Error("NEXT_PUBLIC_IPFS_AUTH is not set");

export const ipfsClient = create({
  url: process.env.NEXT_PUBLIC_IPFS_ENDPOINT,
  headers: {
    Authorization: `Basic ${Buffer.from(authKey).toString("base64")}`,
  },
});

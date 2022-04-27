import create from "zustand";
import { create as createHttp } from "ipfs-http-client";
import { IPFS } from "ipfs-core";

const IPFS_HTTP_ENDPOINT = "https://ipfs.infura.io:5001";

export interface IpfsStore {
  ipfs: IPFS;
}

export const useIpfsStore = create<IpfsStore>(() => ({
  //@ts-ignore
  ipfs: createHttp(IPFS_HTTP_ENDPOINT),
}));

import { IPFS } from "ipfs-core";
import { create as createHttp } from "ipfs-http-client";
import create from "zustand";

const IPFS_HTTP_ENDPOINT = "https://ipfs.infura.io:5001";

export interface IpfsStore {
  ipfs: IPFS;
}

export const useIpfsStore = create<IpfsStore>(() => ({
  //@ts-ignore
  ipfs: createHttp(IPFS_HTTP_ENDPOINT),
}));

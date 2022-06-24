import { IPFSHTTPClient, create as createHttp } from "ipfs-http-client";
import create from "zustand";

export const IPFS_HTTP_ENDPOINT = "https://ipfs.infura.io:5001";

export interface IpfsStore {
  ipfs: IPFSHTTPClient;
}

export const useIpfsStore = create<IpfsStore>(() => ({
  //@ts-ignore
  ipfs: createHttp(IPFS_HTTP_ENDPOINT),
}));

import { ethers } from "ethers";
import create from "zustand";

export interface EthersStore {
  address: string;
  signer: ethers.providers.JsonRpcSigner;
}

export const useEthersStore = create<EthersStore>(() => ({
  address: undefined,
  signer: undefined,
}));

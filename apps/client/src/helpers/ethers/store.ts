import { ethers } from "ethers";
import create from "zustand";

export interface EthersStore {
  address: string | undefined;
  signer: ethers.providers.JsonRpcSigner | undefined;
}

export const useEthersStore = create<EthersStore>(() => ({
  address: undefined,
  signer: undefined,
}));

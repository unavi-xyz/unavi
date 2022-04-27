import { ethers } from "ethers";
import create from "zustand";

export interface LensStore {
  authenticated: boolean;
  connected: boolean;
  signer: undefined | ethers.providers.JsonRpcSigner;
}

export const useStore = create<LensStore>(() => ({
  authenticated: false,
  connected: false,
  signer: undefined,
}));

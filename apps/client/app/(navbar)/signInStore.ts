import { create } from "zustand";

export interface ISignInStore {
  open: boolean;
}

export const useSignInStore = create<ISignInStore>(() => ({
  open: false,
}));

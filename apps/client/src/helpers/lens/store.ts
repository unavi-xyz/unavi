import create from "zustand";

export interface LensStore {
  authenticated: boolean;
  handle: undefined | string;
}

export const useLensStore = create<LensStore>(() => ({
  authenticated: false,
  handle: undefined,
}));

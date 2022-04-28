import create from "zustand";

export interface LensStore {
  authenticated: boolean;
  handle: string | undefined;
}

export const useLensStore = create<LensStore>(() => ({
  authenticated: false,
  handle: undefined,
}));

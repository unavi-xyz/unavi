import { Engine } from "@wired-labs/engine";
import create from "zustand";

export interface AppStore {
  engine: Engine | null;
}

export const useAppStore = create<AppStore>(() => ({
  engine: null,
}));

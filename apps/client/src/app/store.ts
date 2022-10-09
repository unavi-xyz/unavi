import { Engine } from "@wired-labs/engine";
import create from "zustand";

export interface AppStore {
  engine: Engine | null;
  chatBoxFocused: boolean;
}

export const useAppStore = create<AppStore>(() => ({
  engine: null,
  chatBoxFocused: false,
}));

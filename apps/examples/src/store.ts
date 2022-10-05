import { Engine } from "@wired-labs/engine";
import create from "zustand";

export interface IStore {
  engine: Engine | null;
  uri: string | null;
}

export const useStore = create<IStore>(() => ({
  engine: null,
  uri: null,
}));

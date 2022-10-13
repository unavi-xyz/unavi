import { Engine } from "@wired-labs/engine";
import create from "zustand";

export interface AppStore {
  engine: Engine | null;
  chatBoxFocused: boolean;

  displayName: string | null;
  customAvatar: string | null;

  didChangeAvatar: boolean;
}

export const useAppStore = create<AppStore>(() => ({
  engine: null,
  chatBoxFocused: false,
  displayName: null,
  customAvatar: null,

  didChangeAvatar: false,
}));

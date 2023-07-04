import { create } from "zustand";

export interface PlayStore {
  uiAvatar: string;
  chatBoxFocused: boolean;
  uiName: string;
}

export const usePlayStore = create<PlayStore>(() => ({
  chatBoxFocused: false,
  uiAvatar: "",
  uiName: "",
}));

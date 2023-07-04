import { create } from "zustand";

export interface PlayStore {
  avatar: string;
  chatBoxFocused: boolean;
  errorLoading: string | null;
  name: string;
}

export const usePlayStore = create<PlayStore>(() => ({
  avatar: "",
  chatBoxFocused: false,
  errorLoading: null,
  name: "",
}));

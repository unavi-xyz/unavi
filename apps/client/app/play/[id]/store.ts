import { create } from "zustand";

export interface PlayStore {
  nickname: string | null;
  avatar: string | null;

  didChangeName: boolean;
  didChangeAvatar: boolean;
  chatBoxFocused: boolean;

  errorLoading: string | null;
}

export const usePlayStore = create<PlayStore>(() => ({
  nickname: null,
  avatar: null,

  didChangeName: false,
  didChangeAvatar: false,
  chatBoxFocused: false,

  errorLoading: null,
}));

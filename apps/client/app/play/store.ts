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
  avatar: null,
  chatBoxFocused: false,

  didChangeAvatar: false,
  didChangeName: false,
  errorLoading: null,

  nickname: null,
}));

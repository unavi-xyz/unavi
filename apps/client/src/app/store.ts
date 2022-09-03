import { RefObject, createRef } from "react";
import create from "zustand";

export interface AppStore {
  chatInputRef: RefObject<HTMLInputElement>;

  identity: any;
  players: { [id: string]: any };

  messages: any[];
  addMessage: (message: any) => void;
}

export const useAppStore = create<AppStore>((set, get) => ({
  chatInputRef: createRef(),

  identity: {
    isGuest: true,
  },
  players: {},

  messages: [],
  addMessage: (message: any) => {
    const messages = [...get().messages];

    //remove the oldest message if the list is too big
    if (messages.length >= 50) {
      messages.shift();
    }

    set({ messages: [message, ...messages] });
  },
}));

import { RefObject, createRef } from "react";
import create from "zustand";

import { Identity, Message } from "./types";

export interface AppStore {
  chatInputRef: RefObject<HTMLInputElement>;

  identity: Identity;
  players: { [id: string]: Identity };

  messages: Message[];
  addMessage: (message: Message) => void;
}

export const useAppStore = create<AppStore>((set, get) => ({
  chatInputRef: createRef(),

  identity: {
    isGuest: true,
  },
  players: {},

  messages: [],
  addMessage: (message: Message) => {
    const messages = get().messages;

    //remove the oldest message if the list is full
    if (messages.length >= 100) {
      messages.shift();
    }

    set({ messages: [message, ...messages] });
  },
}));

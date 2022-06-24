import { RefObject, createRef } from "react";
import create from "zustand";

import { IChatMessage, PlayerIdentity } from "@wired-xr/engine";

export interface AppStore {
  chatInputRef: RefObject<HTMLInputElement>;

  identity: PlayerIdentity;
  players: { [id: string]: PlayerIdentity };

  messages: IChatMessage[];
  addMessage: (message: IChatMessage) => void;
}

export const useAppStore = create<AppStore>((set, get) => ({
  chatInputRef: createRef(),

  identity: {
    isGuest: true,
  },
  players: {},

  messages: [],
  addMessage: (message: IChatMessage) => {
    const messages = [...get().messages];

    //remove the oldest message if the list is too big
    if (messages.length >= 50) {
      messages.shift();
    }

    set({ messages: [message, ...messages] });
  },
}));

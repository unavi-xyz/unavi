import { RefObject, createRef } from "react";
import create from "zustand";

import { Message } from "./types";

export interface AppStore {
  chatInputRef: RefObject<HTMLInputElement>;
  messages: Message[];
  addMessage: (message: Message) => void;
}

export const useAppStore = create<AppStore>((set, get) => ({
  chatInputRef: createRef(),
  messages: [],
  addMessage: (message: Message) => {
    const messages = get().messages;
    set({ messages: [message, ...messages] });
  },
}));

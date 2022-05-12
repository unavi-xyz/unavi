import create from "zustand";
import { Message } from "./types";

export interface AppStore {
  messages: Message[];
  addMessage: (message: Message) => void;
}

export const useAppStore = create<AppStore>((set, get) => ({
  messages: [],
  addMessage: (message: Message) => {
    const messages = get().messages;
    set({ messages: [...messages, message] });
  },
}));

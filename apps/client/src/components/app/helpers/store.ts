import { MutableRefObject } from "react";
import create from "zustand";

import { AppManager } from "./classes/AppManager";
import { Message } from "./types";

export interface AppStore {
  chatInputRef: MutableRefObject<HTMLInputElement> | undefined;
  messages: Message[];
}

export const useStore = create<AppStore>(() => ({
  chatInputRef: undefined,
  messages: [],
}));

export const appManager = new AppManager(useStore);

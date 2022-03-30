import { MutableRefObject } from "react";
import create from "zustand";

import { AppManager } from "./classes/AppManager";
import { Message } from "./types";

export interface AppStore {
  chatInputRef: MutableRefObject<HTMLInputElement> | undefined;
  messages: Message[];
  muted: boolean;
  spaceId: string;
  isPointerLocked: boolean;
}

export const useStore = create<AppStore>(() => ({
  chatInputRef: undefined,
  messages: [],
  muted: true,
  spaceId: undefined,
  isPointerLocked: false,
}));

export const appManager = new AppManager(useStore);

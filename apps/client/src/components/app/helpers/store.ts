import { MutableRefObject } from "react";
import create from "zustand";

import { AppManager } from "./classes/AppManager";
import { Identity, Message } from "./types";

export interface AppStore {
  chatInputRef: MutableRefObject<HTMLInputElement> | undefined;
  messages: Message[];
  muted: boolean;
  spaceId: string;
  isPointerLocked: boolean;
  players: { [id: string]: Identity };
  identity: Identity;
  connections: RTCPeerConnection[];
}

export const useStore = create<AppStore>(() => ({
  chatInputRef: undefined,
  messages: [],
  muted: true,
  spaceId: undefined,
  isPointerLocked: false,
  players: {},
  identity: undefined,
  connections: [],
}));

export const appManager = new AppManager(useStore);

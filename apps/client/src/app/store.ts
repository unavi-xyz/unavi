import { Engine } from "engine";
import create from "zustand";

import { ChatMessage } from "./ui/ChatMessage";

export interface AppStore {
  engine: Engine | null;
  ws: WebSocket | null;

  playerId: number | null;
  displayName: string | null;
  customAvatar: string | null;

  chatBoxFocused: boolean;
  didChangeName: boolean;
  didChangeAvatar: boolean;

  chatMessages: ChatMessage[];
}

export const useAppStore = create<AppStore>(() => ({
  engine: null,
  ws: null,

  playerId: null,
  displayName: null,
  customAvatar: null,

  chatBoxFocused: false,
  didChangeName: false,
  didChangeAvatar: false,

  chatMessages: [],
}));

import { RequestMessage } from "@wired-protocol/types";
import { Engine } from "lattice-engine/core";
import { create } from "zustand";

import { ChatMessage, EcsEvent } from "./types";

export interface IClientStore {
  sendWebRTC: (message: ArrayBuffer) => void;
  sendWebSockets: (message: RequestMessage) => void;
  cleanupConnection: () => void;
  addChatMessage: (message: ChatMessage) => void;
  worldUri: string;
  skybox: string;
  defaultAvatar: string;
  engine: Engine | null;
  events: EcsEvent[];
  locations: Map<number, number[]>;
  lastLocationUpdates: Map<number, number>;
  grounded: Map<number, boolean>;
  playerId: number | null;
  chatMessages: ChatMessage[];
}

export const useClientStore = create<IClientStore>((set, get) => ({
  addChatMessage: (message: ChatMessage) => {
    const chatMessages = get().chatMessages;
    chatMessages.push(message);
    if (chatMessages.length > 100) chatMessages.shift();
    set({ chatMessages: [...chatMessages] });
  },
  chatMessages: [],
  cleanupConnection: () => {},
  defaultAvatar: "",
  engine: null,
  events: [],
  grounded: new Map(),
  lastLocationUpdates: new Map(),
  locations: new Map(),
  playerId: null,
  sendWebRTC: () => {},
  sendWebSockets: () => {},
  skybox: "",
  worldUri: "",
}));

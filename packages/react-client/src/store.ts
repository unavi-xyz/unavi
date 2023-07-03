import { RequestMessage } from "@wired-protocol/types";
import { Engine } from "lattice-engine/core";
import { create } from "zustand";

import { ChatMessage, EcsEvent } from "./types";

export interface IClientStore {
  sendWebRTC: (message: ArrayBuffer) => void;
  sendWebSockets: (message: RequestMessage) => void;
  cleanupConnection: () => void;
  addChatMessage: (message: ChatMessage) => void;
  avatar: string;
  avatars: Map<number, string>;
  chatMessages: ChatMessage[];
  defaultAvatar: string;
  engine: Engine | null;
  events: EcsEvent[];
  grounded: Map<number, boolean>;
  handle: string;
  lastLocationUpdates: Map<number, number>;
  locations: Map<number, number[]>;
  name: string;
  playerId: number | null;
  skybox: string;
  worldUri: string;
}

export const useClientStore = create<IClientStore>((set, get) => ({
  addChatMessage: (message: ChatMessage) => {
    const chatMessages = get().chatMessages;
    chatMessages.push(message);
    if (chatMessages.length > 100) chatMessages.shift();
    set({ chatMessages: [...chatMessages] });
  },
  avatar: "",
  avatars: new Map(),
  chatMessages: [],
  cleanupConnection: () => {},
  defaultAvatar: "",
  engine: null,
  events: [],
  grounded: new Map(),
  handle: "",
  lastLocationUpdates: new Map(),
  locations: new Map(),
  name: "",
  playerId: null,
  sendWebRTC: () => {},
  sendWebSockets: () => {},
  skybox: "",
  worldUri: "",
}));

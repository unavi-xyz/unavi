import { RequestMessage } from "@wired-protocol/types";
import { create } from "zustand";

export type PlayerMessage = {
  type: "player";
  id: number;
  timestamp: number;
  text: string;
  sender: string;
};

export type SystemMessage = {
  type: "system";
  id: number;
  timestamp: number;
  text: string;
};

export type ChatMessage = PlayerMessage | SystemMessage;

export interface IClientStore {
  sendWebRTC: (message: ArrayBuffer) => void;
  sendWebSockets: (message: RequestMessage) => void;
  cleanupConnection: () => void;
  addChatMessage: (message: ChatMessage) => void;
  worldUri: string;
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
  playerId: null,
  sendWebRTC: () => {},
  sendWebSockets: () => {},
  worldUri: "",
}));

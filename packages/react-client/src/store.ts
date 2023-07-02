import { RequestMessage } from "@wired-protocol/types";
import { create } from "zustand";

export interface IClientStore {
  sendWebRTC: (message: ArrayBuffer) => void;
  sendWebSockets: (message: RequestMessage) => void;
  cleanupConnection: () => void;
  worldUri: string;
  playerId: number | null;
}

export const useClientStore = create<IClientStore>(() => ({
  cleanupConnection: () => {},
  playerId: null,
  sendWebRTC: () => {},
  sendWebSockets: () => {},
  worldUri: "",
}));

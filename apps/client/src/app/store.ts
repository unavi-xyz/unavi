import { Engine } from "engine";
import { Producer } from "mediasoup-client/lib/Producer";
import { Transport } from "mediasoup-client/lib/Transport";
import create from "zustand";

import { ChatMessage } from "./ui/ChatMessage";

export interface AppStore {
  engine: Engine | null;
  ws: WebSocket | null;

  producer: Producer | null;
  producerTransport: Transport | null;
  producedTrack: MediaStreamTrack | null;
  micPaused: boolean;

  playerId: number | null;
  displayName: string | null;
  customAvatar: string | null;

  didChangeName: boolean;
  didChangeAvatar: boolean;
  chatBoxFocused: boolean;
  chatMessages: ChatMessage[];
}

export const useAppStore = create<AppStore>(() => ({
  engine: null,
  ws: null,

  producer: null,
  producerTransport: null,
  producedTrack: null,
  micPaused: false,

  playerId: null,
  displayName: null,
  customAvatar: null,

  didChangeName: false,
  didChangeAvatar: false,
  chatBoxFocused: false,
  chatMessages: [],
}));

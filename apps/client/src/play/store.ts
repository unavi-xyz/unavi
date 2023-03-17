import { Engine } from "engine";
import { Producer } from "mediasoup-client/lib/Producer";
import { Transport } from "mediasoup-client/lib/Transport";
import { create } from "zustand";

import { Players } from "./networking/Players";
import { ChatMessage } from "./ui/ChatMessage";

export interface PlayStore {
  engine: Engine | null;
  ws: WebSocket | null;
  players: Players | null;

  producer: Producer | null;
  producerTransport: Transport | null;
  producedTrack: MediaStreamTrack | null;
  micPaused: boolean;

  playerId: number | null;
  nickname: string | null;
  avatar: string | null;

  didChangeName: boolean;
  didChangeAvatar: boolean;
  chatBoxFocused: boolean;
  chatMessages: ChatMessage[];

  errorLoading: string | null;
}

export const usePlayStore = create<PlayStore>(() => ({
  engine: null,
  ws: null,
  players: null,

  producer: null,
  producerTransport: null,
  producedTrack: null,
  micPaused: false,

  playerId: null,
  nickname: null,
  avatar: null,

  didChangeName: false,
  didChangeAvatar: false,
  chatBoxFocused: false,
  chatMessages: [],

  errorLoading: null,
}));

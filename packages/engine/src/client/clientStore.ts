import { Event, Response, SendEvent } from "@wired-protocol/types";
import { Engine } from "lattice-engine/core";
import { create } from "zustand";

import { ChatMessage } from "./types";
import { splitHandle } from "./utils/splitHandle";
import { toHex } from "./utils/toHex";

export interface IClientStore {
  addChatMessage: (message: ChatMessage) => void;
  cleanupConnection: () => void;
  getDisplayName: (playerId: number) => string;
  sendWebRTC: (message: ArrayBuffer) => void;
  sendWebSockets: (message: Uint8Array) => void;
  setAvatar: (avatar: string) => void;
  setHandle: (handle: string) => void;
  setName: (name: string) => void;
  setPlayerData: (playerId: number, key: string, value: string) => void;
  setPlayerId: (playerId: number | null) => void;
  mirrorEvent: (data: Uint8Array) => void;
  avatar: string;
  chatMessages: ChatMessage[];
  defaultAvatar: string;
  engine: Engine | null;
  ecsIncoming: Response[];
  exportedModel: Blob | null;
  playerData: Map<number, Record<string, string>>;
  handle: string;
  lastLocationUpdates: Map<number, number>;
  locations: Map<number, number[]>;
  nickname: string;
  playerId: number | null;
  rootName: string;
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
  chatMessages: [],
  cleanupConnection: () => { },
  defaultAvatar: "",
  ecsIncoming: [],
  engine: null,
  exportedModel: null,
  getDisplayName: (playerId: number) => {
    const playerData = get().playerData.get(playerId);

    const handle = playerData?.handle;
    const nickname = playerData?.nickname;

    if (handle) {
      const { username } = splitHandle(handle);

      if (username) {
        return `@${username}`;
      }
    }

    if (nickname) {
      return nickname;
    }

    return `Guest ${toHex(playerId)}`;
  },
  handle: "",
  lastLocationUpdates: new Map(),
  locations: new Map(),
  mirrorEvent: (data: Uint8Array) => {
    // Send to self
    const playerId = get().playerId;
    if (playerId === null) return;

    const event = Event.create({ data, playerId });
    const response = Response.create({
      response: { event, oneofKind: "event" },
    });
    const events = get().ecsIncoming;
    events.push(response);

    // Send to others
    const sendEvent = SendEvent.create({ data });
    get().sendWebSockets(SendEvent.toBinary(sendEvent));
  },
  nickname: "",
  playerData: new Map(),
  playerId: null,
  rootName: "",
  sendWebRTC: () => { },
  sendWebSockets: () => { },
  setAvatar(avatar: string) {
    set({ avatar });

    const playerId = get().playerId;
    if (playerId === null) return;

    get().setPlayerData(playerId, "avatar", avatar);
  },
  setHandle(handle: string) {
    set({ handle });

    const playerId = get().playerId;
    if (playerId === null) return;

    get().setPlayerData(playerId, "handle", handle);
  },
  setName(nickname: string) {
    set({ nickname });

    const playerId = get().playerId;
    if (playerId === null) return;

    get().setPlayerData(playerId, "nickname", nickname);
  },
  setPlayerData(playerId: number, key: string, value: string) {
    const playerData = get().playerData.get(playerId) || {};
    playerData[key] = value;
    get().playerData.set(playerId, playerData);
  },
  setPlayerId(playerId: number | null) {
    const oldPlayerId = get().playerId;

    if (oldPlayerId === playerId) return;

    if (oldPlayerId !== null) {
      get().playerData.delete(oldPlayerId);
    }

    if (playerId !== null) {
      get().setPlayerData(playerId, "nickname", get().nickname);
      get().setPlayerData(playerId, "handle", get().handle);
      get().setPlayerData(playerId, "avatar", get().avatar);
    }

    set({ playerId });
  },
  skybox: "",
  worldUri: "",
}));

import { EditorEvent } from "@unavi/protocol";
import { Event, Request, Response, SendEvent } from "@wired-protocol/types";
import { Engine } from "houseki/core";
import { create } from "zustand";

import { ChatMessage } from "./types";
import { toHex } from "./utils/toHex";

export interface IClientStore {
  addChatMessage: (message: ChatMessage) => void;
  cleanupConnection: () => void;
  getDisplayName: (playerId: number) => string;
  sendWebRTC: (message: ArrayBuffer) => void;
  sendWebSockets: (message: Request["message"]) => void;
  setAvatar: (avatar: string) => void;
  setDID: (did: string) => void;
  setName: (name: string) => void;
  setPlayerData: (playerId: number, key: string, value: string) => void;
  setPlayerId: (playerId: number | null) => void;
  mirrorEvent: (editorEvent: EditorEvent) => void;
  avatar: string;
  chatMessages: ChatMessage[];
  defaultAvatar: string;
  engine: Engine | null;
  ecsIncoming: Response[];
  exportedModel: Blob | null;
  playerData: Map<number, Record<string, string>>;
  did: string;
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
  cleanupConnection: () => {},
  defaultAvatar: "",
  did: "",
  ecsIncoming: [],
  engine: null,
  exportedModel: null,
  getDisplayName: (playerId: number) => {
    const playerData = get().playerData.get(playerId);

    const did = playerData?.did;
    const nickname = playerData?.nickname;

    if (did) {
      // TODO: Resolve profile
    }

    if (nickname) {
      return nickname;
    }

    return `Guest ${toHex(playerId)}`;
  },
  lastLocationUpdates: new Map(),
  locations: new Map(),
  mirrorEvent: (editorEvent: EditorEvent) => {
    const data = EditorEvent.toBinary(editorEvent);

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
    get().sendWebSockets({ oneofKind: "sendEvent", sendEvent });
  },
  nickname: "",
  playerData: new Map(),
  playerId: null,
  rootName: "",
  sendWebRTC: () => {},
  sendWebSockets: () => {},
  setAvatar(avatar: string) {
    set({ avatar });

    const playerId = get().playerId;
    if (playerId === null) return;

    get().setPlayerData(playerId, "avatar", avatar);
  },
  setDID(did: string) {
    set({ did });

    const playerId = get().playerId;
    if (playerId === null) return;

    get().setPlayerData(playerId, "did", did);
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
      get().setPlayerData(playerId, "did", get().did);
      get().setPlayerData(playerId, "avatar", get().avatar);
    }

    set({ playerId });
  },
  skybox: "",
  worldUri: "",
}));

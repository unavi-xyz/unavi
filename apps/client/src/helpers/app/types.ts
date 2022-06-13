import { Triplet } from "@react-three/cannon";

export type IChatMessage = {
  id: string;
  username: string;
  message: string;
  timestamp: number;
};

export type PlayerIdentity = {
  isGuest: boolean;
  handle?: string;
};

export type PlayerLocation = {
  position: Triplet;
  rotation: number;
};

export type Channels = {
  message: string;
  identity: PlayerIdentity;
  location: PlayerLocation;
};

export const channelNames: Array<keyof Channels> = [
  "identity",
  "message",
  "location",
];

export type PlayerChannels = Record<keyof Channels, RTCDataChannel>;

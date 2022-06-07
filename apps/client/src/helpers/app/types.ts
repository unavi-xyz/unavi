import { Triplet } from "@react-three/cannon";

export type Message = {
  id: string;
  username: string;
  text: string;
  time: number;
};

export type Identity = {
  isGuest: boolean;
  handle?: string;
};

export type Location = {
  position: Triplet;
  rotation: number;
};

export type Channels = {
  message: string;
  identity: Identity;
  location: Location;
};

export const channelNames: Array<keyof Channels> = [
  "identity",
  "message",
  "location",
];

export type PlayerChannels = Record<keyof Channels, RTCDataChannel>;

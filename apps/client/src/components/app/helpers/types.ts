import { Triplet } from "@react-three/cannon";

export type Message = {
  id: string;
  username: string;
  text: string;
  time: number;
};

export type Identity = {
  isGuest: boolean;
  did: string;
};

export type Transform = {
  position: Triplet;
  rotation: number;
};

export type Channels = {
  message: string;
  identity: Identity;
  transform: Transform;
};

export const channelNames: Array<keyof Channels> = [
  "identity",
  "message",
  "transform",
];

export type PlayerChannels = Partial<Record<keyof Channels, RTCDataChannel>>;

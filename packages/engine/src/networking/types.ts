import { Triplet } from "@react-three/cannon";

import {
  ConnectTransportData,
  ConsumeAudioData,
  ConsumeAudioResponse,
  CreateTransportResponse,
  GetRouterRtpCapabilitiesResponse,
  JoinSpaceData,
  JoinSpaceResponse,
  LeaveSpaceData,
  LeaveSpaceResponse,
  NewConsumerData,
  ProduceAudioData,
  ProduceAudioResponse,
} from "./schemas";

export interface SocketEvents {
  join_space: (
    data: JoinSpaceData,
    callback: (res: JoinSpaceResponse) => void | Promise<void>
  ) => void | Promise<void>;
  leave_space: (
    data: LeaveSpaceData,
    callback: (res: LeaveSpaceResponse) => void | Promise<void>
  ) => void | Promise<void>;
  get_router_rtp_capabilities: (
    callback: (res: GetRouterRtpCapabilitiesResponse) => void
  ) => void | Promise<void>;
  create_audio_producer_transport: (
    callback: (res: CreateTransportResponse) => void
  ) => void | Promise<void>;
  create_audio_consumer_transport: (
    callback: (res: CreateTransportResponse) => void
  ) => void | Promise<void>;
  connect_audio_producer_transport: (
    data: ConnectTransportData,
    callback: (res: CreateTransportResponse) => void
  ) => void | Promise<void>;
  connect_audio_consumer_transport: (
    data: ConnectTransportData,
    callback: (res: CreateTransportResponse) => void
  ) => void | Promise<void>;
  produce_audio: (
    data: ProduceAudioData,
    callback: (res: ProduceAudioResponse) => void
  ) => void | Promise<void>;
  consume_audio: (
    data: ConsumeAudioData,
    callback: (res: ConsumeAudioResponse) => void
  ) => void | Promise<void>;
  new_consumer: (data: NewConsumerData) => void | Promise<void>;
}

//old
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

export type MessageType = "chatmessage" | "identity" | "location" | "leave";

export type WebsocketMessageBase<T extends MessageType, D> = {
  type: T;
  data: D;
};

export type SentChatMessage = WebsocketMessageBase<"chatmessage", string>;
export type SentIdentity = WebsocketMessageBase<"identity", string>;
export type SentLocation = WebsocketMessageBase<"location", PlayerLocation>;

export type SentWebsocketMessage =
  | SentChatMessage
  | SentIdentity
  | SentLocation;

export type RecievedChatMessage = WebsocketMessageBase<
  "chatmessage",
  IChatMessage
>;
export type RecievedIdentity = WebsocketMessageBase<"identity", PlayerIdentity>;
export type RecievedLocation = WebsocketMessageBase<
  "location",
  {
    userid: string;
    handle?: string;
    location: PlayerLocation;
  }
>;
export type RecievedLeaveMessage = WebsocketMessageBase<
  "leave",
  {
    userid: string;
  }
>;

export type RecievedWebsocketMessage =
  | RecievedChatMessage
  | RecievedIdentity
  | RecievedLocation
  | RecievedLeaveMessage;

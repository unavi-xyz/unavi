import { IChatMessage, PlayerIdentity, PlayerLocation } from "../app/types";

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

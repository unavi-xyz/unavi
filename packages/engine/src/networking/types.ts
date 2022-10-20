import {
  RtpCapabilities,
  RtpParameters,
} from "mediasoup-client/lib/RtpParameters";
import { SctpStreamParameters } from "mediasoup-client/lib/SctpParameters";
import { TransportOptions } from "mediasoup-client/lib/Transport";

export type InternalChatMessage = {
  id: string;
  playerId: string;
  username: string;
  message: string;
  timestamp: number;
  isHandle: boolean;
};

export type IChatMessage = {
  id: string;
  playerId: string;
  message: string;
  timestamp: number;
};

type GenericWebSocketMessage<S extends string, D> = {
  subject: S;
  data: D;
};

export type ToHostMessage =
  | GenericWebSocketMessage<
      "join",
      {
        spaceId: string;
      }
    >
  | GenericWebSocketMessage<"leave", null>
  | GenericWebSocketMessage<
      "location",
      [number, number, number, number, number, number, number]
    >
  | GenericWebSocketMessage<"message", string>
  | GenericWebSocketMessage<"falling_state", boolean>
  | GenericWebSocketMessage<"set_name", string | null>
  | GenericWebSocketMessage<"set_avatar", string | null>
  | GenericWebSocketMessage<"set_handle", string | null>
  | GenericWebSocketMessage<"get_router_rtp_capabilities", null>
  | GenericWebSocketMessage<
      "create_transport",
      {
        type: "producer" | "consumer";
      }
    >
  | GenericWebSocketMessage<
      "connect_transport",
      {
        dtlsParameters: TransportOptions["dtlsParameters"];
        type: "producer" | "consumer";
      }
    >
  | GenericWebSocketMessage<
      "produce",
      {
        rtpParameters: RtpParameters;
      }
    >
  | GenericWebSocketMessage<
      "produce_data",
      {
        sctpStreamParameters: SctpStreamParameters;
      }
    >
  | GenericWebSocketMessage<
      "set_rtp_capabilities",
      {
        rtpCapabilities: RtpCapabilities;
      }
    >;

export type FromHostMessage =
  | GenericWebSocketMessage<
      "join_successful",
      {
        playerId: string;
      }
    >
  | GenericWebSocketMessage<
      "player_joined",
      {
        playerId: string;
        name: string | null;
        avatar: string | null;
        handle: string | null;
      }
    >
  | GenericWebSocketMessage<"player_left", string>
  | GenericWebSocketMessage<
      "player_location",
      {
        playerId: string;
        location: [number, number, number, number, number, number, number];
      }
    >
  | GenericWebSocketMessage<"player_message", IChatMessage>
  | GenericWebSocketMessage<
      "player_falling_state",
      {
        playerId: string;
        isFalling: boolean;
      }
    >
  | GenericWebSocketMessage<
      "player_name",
      { playerId: string; name: string | null }
    >
  | GenericWebSocketMessage<
      "player_avatar",
      { playerId: string; avatar: string | null }
    >
  | GenericWebSocketMessage<
      "player_handle",
      { playerId: string; handle: string | null }
    >
  | GenericWebSocketMessage<"router_rtp_capabilities", RtpCapabilities>
  | GenericWebSocketMessage<
      "transport_created",
      {
        type: "producer" | "consumer";
        options: TransportOptions;
      }
    >
  | GenericWebSocketMessage<
      "create_consumer",
      {
        id: string;
        producerId: string;
        rtpParameters: RtpParameters;
      }
    >
  | GenericWebSocketMessage<
      "create_data_consumer",
      {
        id: string;
        dataProducerId: string;
        sctpStreamParameters: SctpStreamParameters;
      }
    >
  | GenericWebSocketMessage<
      "producer_id",
      {
        id: string;
      }
    >
  | GenericWebSocketMessage<
      "data_producer_id",
      {
        id: string;
      }
    >;

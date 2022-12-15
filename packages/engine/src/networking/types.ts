import { RtpCapabilities, RtpParameters } from "mediasoup-client/lib/RtpParameters";
import { SctpStreamParameters } from "mediasoup-client/lib/SctpParameters";
import { TransportOptions } from "mediasoup-client/lib/Transport";

export type InternalChatMessage =
  | {
      type: "chat";
      id: string;
      timestamp: number;
      playerId: number;
      username: string;
      message: string;
      isHandle: boolean;
    }
  | {
      type: "system";
      variant: "player_joined" | "player_left";
      id: string;
      timestamp: number;
      playerId: number;
      username: string;
      isHandle: boolean;
    };

export type IChatMessage = {
  id: string;
  playerId: number;
  message: string;
  timestamp: number;
};

export type SpaceJoinStatus = {
  spaceId: string | null;
  spaceFetched: boolean;
  wsConnected: boolean;
  webrtcConnected: boolean;
  sceneLoaded: boolean;
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
  | GenericWebSocketMessage<"location", [number, number, number, number, number, number, number]>
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
    >
  | GenericWebSocketMessage<"ready_to_consume", boolean>
  | GenericWebSocketMessage<"resume_audio", null>;

export type FromHostMessage =
  | GenericWebSocketMessage<
      "join_successful",
      {
        playerId: number;
      }
    >
  | GenericWebSocketMessage<
      "player_joined",
      {
        playerId: number;
        name: string | null;
        avatar: string | null;
        handle: string | null;
        beforeYou?: boolean;
      }
    >
  | GenericWebSocketMessage<"player_left", number>
  | GenericWebSocketMessage<"player_message", IChatMessage>
  | GenericWebSocketMessage<
      "player_falling_state",
      {
        playerId: number;
        isFalling: boolean;
      }
    >
  | GenericWebSocketMessage<"player_name", { playerId: number; name: string | null }>
  | GenericWebSocketMessage<"player_avatar", { playerId: number; avatar: string | null }>
  | GenericWebSocketMessage<"player_handle", { playerId: number; handle: string | null }>
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
        playerId: number;
        id: string;
        producerId: string;
        rtpParameters: RtpParameters;
      }
    >
  | GenericWebSocketMessage<
      "create_data_consumer",
      {
        playerId: number;
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

import { RtpCapabilities, RtpParameters } from "mediasoup-client/lib/RtpParameters";
import { SctpStreamParameters } from "mediasoup-client/lib/SctpParameters";
import { TransportOptions } from "mediasoup-client/lib/Transport";

export type ChatMessage = {
  id: string;
  playerId: number;
  message: string;
  timestamp: number;
};

type GenericMessage<S extends string, D> = {
  subject: S;
  data: D;
};

export type ToHostMessage =
  | GenericMessage<"join", { id: number }>
  | GenericMessage<"leave", null>
  | GenericMessage<"location", [number, number, number, number, number, number, number]>
  | GenericMessage<"message", string>
  | GenericMessage<"falling_state", boolean>
  | GenericMessage<"set_name", string | null>
  | GenericMessage<"set_avatar", string | null>
  | GenericMessage<"set_address", string | null>
  | GenericMessage<"get_router_rtp_capabilities", null>
  | GenericMessage<"create_transport", { type: "producer" | "consumer" }>
  | GenericMessage<
      "connect_transport",
      {
        dtlsParameters: TransportOptions["dtlsParameters"];
        type: "producer" | "consumer";
      }
    >
  | GenericMessage<"produce", { rtpParameters: RtpParameters }>
  | GenericMessage<"produce_data", { sctpStreamParameters: SctpStreamParameters }>
  | GenericMessage<"set_rtp_capabilities", { rtpCapabilities: RtpCapabilities }>
  | GenericMessage<"ready_to_consume", boolean>
  | GenericMessage<"resume_audio", null>;

export type FromHostMessage =
  | GenericMessage<"join_successful", { playerId: number }>
  | GenericMessage<
      "player_joined",
      {
        playerId: number;
        name: string | null;
        avatar: string | null;
        address: string | null;
        beforeYou?: boolean;
      }
    >
  | GenericMessage<"player_left", number>
  | GenericMessage<"player_message", ChatMessage>
  | GenericMessage<"player_falling_state", { playerId: number; isFalling: boolean }>
  | GenericMessage<"player_name", { playerId: number; name: string | null }>
  | GenericMessage<"player_avatar", { playerId: number; avatar: string | null }>
  | GenericMessage<"player_address", { playerId: number; address: string | null }>
  | GenericMessage<"router_rtp_capabilities", RtpCapabilities>
  | GenericMessage<
      "transport_created",
      { type: "producer" | "consumer"; options: TransportOptions }
    >
  | GenericMessage<
      "create_consumer",
      {
        playerId: number;
        id: string;
        producerId: string;
        rtpParameters: RtpParameters;
      }
    >
  | GenericMessage<
      "create_data_consumer",
      {
        playerId: number;
        id: string;
        dataProducerId: string;
        sctpStreamParameters: SctpStreamParameters;
      }
    >
  | GenericMessage<"producer_id", { id: string }>
  | GenericMessage<"data_producer_id", { id: string }>;

import { z } from "zod";

import { MediasoupSchema } from "./mediasoup";

const TO_HOST_SCHEMA = {
  join: z.object({ id: z.number().int().positive() }),
  leave: z.null(),
  chat: z.string(),
  set_falling_state: z.boolean(),
  set_name: z.union([z.string(), z.null()]),
  set_avatar: z.union([z.string(), z.null()]),
  set_address: z.union([z.string(), z.null()]),

  // WebRTC
  get_router_rtp_capabilities: z.null(),
  create_transport: z.object({
    type: z.union([z.literal("producer"), z.literal("consumer")]),
  }),
  connect_transport: z.object({
    type: z.union([z.literal("producer"), z.literal("consumer")]),
    dtlsParameters: MediasoupSchema.dltsParameters,
  }),
  produce: z.object({ rtpParameters: MediasoupSchema.rtpParameters }),
  produce_data: z.object({ sctpStreamParameters: MediasoupSchema.sctpStreamParameters }),
  set_rtp_capabilities: z.object({ rtpCapabilities: MediasoupSchema.rtpCapabilities }),
  ready_to_consume: z.boolean(),
  resume_audio: z.null(),
} as const;

const FROM_HOST_SCHEMA = {
  join_success: z.object({ playerId: z.number().int().positive() }),

  // Players
  player_joined: z.object({
    playerId: z.number().int().positive(),
    name: z.string().nullable(),
    avatar: z.string().nullable(),
    address: z.string().nullable(),
    beforeYou: z.boolean().optional(),
  }),
  player_left: z.object({ playerId: z.number().int().positive() }),
  player_chat: z.object({
    playerId: z.number().int().positive(),
    message: z.string(),
    timestamp: z.number().int().positive(),
  }),
  player_falling_state: z.object({
    playerId: z.number().int().positive(),
    isFalling: z.boolean(),
  }),
  player_name: z.object({
    playerId: z.number().int().positive(),
    name: z.string().nullable(),
  }),
  player_avatar: z.object({
    playerId: z.number().int().positive(),
    avatar: z.string().nullable(),
  }),
  player_address: z.object({
    playerId: z.number().int().positive(),
    address: z.string().nullable(),
  }),

  // WebRTC
  router_rtp_capabilities: z.object({ rtpCapabilities: MediasoupSchema.rtpCapabilities }),
  transport_created: z.object({
    type: z.union([z.literal("producer"), z.literal("consumer")]),
    options: MediasoupSchema.transportOptions,
  }),
  create_consumer: z.object({
    playerId: z.number().int().positive(),
    id: z.string(),
    producerId: z.string(),
    rtpParameters: MediasoupSchema.rtpParameters,
  }),
  create_data_consumer: z.object({
    playerId: z.number().int().positive(),
    id: z.string(),
    dataProducerId: z.string(),
    sctpStreamParameters: MediasoupSchema.sctpStreamParameters,
  }),
  producer_id: z.object({ id: z.string() }),
  data_producer_id: z.object({ id: z.string() }),
} as const;

function getMessageSchema<S extends string, D>(subject: S, data: z.ZodSchema<D>) {
  return z.object({ subject: z.literal(subject), data });
}

export const toHostMessageSchema = z.union([
  getMessageSchema("join", TO_HOST_SCHEMA.join),
  getMessageSchema("leave", TO_HOST_SCHEMA.leave),
  getMessageSchema("chat", TO_HOST_SCHEMA.chat),
  getMessageSchema("set_falling_state", TO_HOST_SCHEMA.set_falling_state),
  getMessageSchema("set_name", TO_HOST_SCHEMA.set_name),
  getMessageSchema("set_avatar", TO_HOST_SCHEMA.set_avatar),
  getMessageSchema("set_address", TO_HOST_SCHEMA.set_address),

  // WebRTC
  getMessageSchema("get_router_rtp_capabilities", TO_HOST_SCHEMA.get_router_rtp_capabilities),
  getMessageSchema("create_transport", TO_HOST_SCHEMA.create_transport),
  getMessageSchema("connect_transport", TO_HOST_SCHEMA.connect_transport),
  getMessageSchema("produce", TO_HOST_SCHEMA.produce),
  getMessageSchema("produce_data", TO_HOST_SCHEMA.produce_data),
  getMessageSchema("set_rtp_capabilities", TO_HOST_SCHEMA.set_rtp_capabilities),
  getMessageSchema("ready_to_consume", TO_HOST_SCHEMA.ready_to_consume),
  getMessageSchema("resume_audio", TO_HOST_SCHEMA.resume_audio),
] as const);

export const fromHostMessageSchema = z.union([
  getMessageSchema("join_success", FROM_HOST_SCHEMA.join_success),

  // Players
  getMessageSchema("player_joined", FROM_HOST_SCHEMA.player_joined),
  getMessageSchema("player_left", FROM_HOST_SCHEMA.player_left),
  getMessageSchema("player_chat", FROM_HOST_SCHEMA.player_chat),
  getMessageSchema("player_falling_state", FROM_HOST_SCHEMA.player_falling_state),
  getMessageSchema("player_name", FROM_HOST_SCHEMA.player_name),
  getMessageSchema("player_avatar", FROM_HOST_SCHEMA.player_avatar),
  getMessageSchema("player_address", FROM_HOST_SCHEMA.player_address),

  // WebRTC
  getMessageSchema("router_rtp_capabilities", FROM_HOST_SCHEMA.router_rtp_capabilities),
  getMessageSchema("transport_created", FROM_HOST_SCHEMA.transport_created),
  getMessageSchema("create_consumer", FROM_HOST_SCHEMA.create_consumer),
  getMessageSchema("create_data_consumer", FROM_HOST_SCHEMA.create_data_consumer),
  getMessageSchema("producer_id", FROM_HOST_SCHEMA.producer_id),
  getMessageSchema("data_producer_id", FROM_HOST_SCHEMA.data_producer_id),
] as const);

export type ToHostMessage = z.infer<typeof toHostMessageSchema>;
export type FromHostMessage = z.infer<typeof fromHostMessageSchema>;

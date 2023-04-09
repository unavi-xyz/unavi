import { z } from "zod";

import { MediasoupSchema } from "./MediasoupSchema";

const spaceURI = z.string();
const playerId = z.number().int().min(0).max(255);

export const FromHostSchema = {
  join_success: z.object({ uri: spaceURI, playerId }),

  // Players
  player_address: z.object({ playerId, address: z.string().nullable() }),
  player_avatar: z.object({ playerId, avatar: z.string().nullable() }),
  player_chat: z.object({
    playerId,
    text: z.string(),
    timestamp: z.number().int().positive(),
  }),
  player_grounded: z.object({ playerId, grounded: z.boolean() }),
  player_joined: z.object({
    playerId,
    name: z.string().nullable(),
    avatar: z.string().nullable(),
    address: z.string().nullable(),
    beforeYou: z.boolean().optional(),
  }),
  player_left: z.object({ playerId }),
  player_name: z.object({ playerId, name: z.string().nullable() }),

  // WebRTC
  create_consumer: z.object({
    playerId,
    consumerId: z.string(),
    producerId: z.string(),
    rtpParameters: MediasoupSchema.rtpParameters,
  }),
  create_data_consumer: z.object({
    playerId,
    consumerId: z.string(),
    dataProducerId: z.string(),
    sctpStreamParameters: MediasoupSchema.sctpStreamParameters,
  }),
  data_producer_id: z.string(),
  producer_id: z.string(),
  router_rtp_capabilities: MediasoupSchema.rtpCapabilities,
  transport_created: z.object({
    type: z.union([z.literal("producer"), z.literal("consumer")]),
    options: MediasoupSchema.transportOptions,
  }),
} as const;

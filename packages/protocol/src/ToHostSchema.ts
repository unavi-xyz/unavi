import { z } from "zod";

import { MediasoupSchema } from "./MediasoupSchema";

const spaceURI = z.string();

export const ToHostSchema = {
  chat: z.string(),
  join: spaceURI,
  leave: spaceURI,
  set_address: z.union([z.string(), z.null()]),
  set_avatar: z.union([z.string(), z.null()]),
  set_grounded: z.boolean(),
  set_name: z.union([z.string(), z.null()]),

  // WebRTC
  connect_transport: z.object({
    type: z.union([z.literal("producer"), z.literal("consumer")]),
    dtlsParameters: MediasoupSchema.dltsParameters,
  }),
  create_transport: z.object({
    type: z.union([z.literal("producer"), z.literal("consumer")]),
  }),
  get_router_rtp_capabilities: z.null(),
  produce: MediasoupSchema.rtpParameters,
  produce_data: MediasoupSchema.sctpStreamParameters,
  resume_audio: z.null(),
  set_rtp_capabilities: MediasoupSchema.rtpCapabilities,
} as const;

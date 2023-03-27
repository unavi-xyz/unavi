import { z } from "zod";

import { FromHostSchema } from "./FromHostSchema";
import { ToHostSchema } from "./ToHostSchema";

function toMessage<S extends string, D>(subject: S, data: z.ZodSchema<D>) {
  return z.object({ subject: z.literal(subject), data });
}

export class MessageSchema {
  static fromHost = z.union([
    toMessage("join_success", FromHostSchema.join_success),

    // Players
    toMessage("player_address", FromHostSchema.player_address),
    toMessage("player_avatar", FromHostSchema.player_avatar),
    toMessage("player_chat", FromHostSchema.player_chat),
    toMessage("player_grounded", FromHostSchema.player_grounded),
    toMessage("player_joined", FromHostSchema.player_joined),
    toMessage("player_left", FromHostSchema.player_left),
    toMessage("player_name", FromHostSchema.player_name),

    // WebRTC
    toMessage("create_consumer", FromHostSchema.create_consumer),
    toMessage("create_data_consumer", FromHostSchema.create_data_consumer),
    toMessage("data_producer_id", FromHostSchema.data_producer_id),
    toMessage("producer_id", FromHostSchema.producer_id),
    toMessage("router_rtp_capabilities", FromHostSchema.router_rtp_capabilities),
    toMessage("transport_created", FromHostSchema.transport_created),
  ]);

  static toHost = z.union([
    toMessage("chat", ToHostSchema.chat),
    toMessage("join", ToHostSchema.join),
    toMessage("leave", ToHostSchema.leave),
    toMessage("set_address", ToHostSchema.set_address),
    toMessage("set_avatar", ToHostSchema.set_avatar),
    toMessage("set_grounded", ToHostSchema.set_grounded),
    toMessage("set_name", ToHostSchema.set_name),

    // WebRTC
    toMessage("connect_transport", ToHostSchema.connect_transport),
    toMessage("create_transport", ToHostSchema.create_transport),
    toMessage("get_router_rtp_capabilities", ToHostSchema.get_router_rtp_capabilities),
    toMessage("produce", ToHostSchema.produce),
    toMessage("produce_data", ToHostSchema.produce_data),
    toMessage("resume_audio", ToHostSchema.resume_audio),
    toMessage("set_rtp_capabilities", ToHostSchema.set_rtp_capabilities),
  ]);
}

export type FromHostMessage = z.infer<typeof MessageSchema.fromHost>;
export type ToHostMessage = z.infer<typeof MessageSchema.toHost>;

import { z } from "zod";

const MediaTypeSchema = z.union([z.literal("audio"), z.literal("video")]);

const RtpCapabilitiesSchema = z.object({
  codecs: z
    .array(
      z.object({
        kind: z.union([z.literal("audio"), z.literal("video")]),
        mimeType: z.string(),
        clockRate: z.number(),
        preferredPayloadType: z.number().optional(),
        channels: z.number().optional(),
        parameters: z.any().optional(),
        rtcpFeedback: z
          .array(
            z.object({
              type: z.string(),
              parameter: z.string().optional(),
            })
          )
          .optional(),
      })
    )
    .optional(),

  headerExtensions: z
    .array(
      z.object({
        kind: z.union([z.literal("audio"), z.literal("video")]),
        uri: z.string(),
        preferredId: z.number(),
        preferredEncrypt: z.boolean().optional(),
        direction: z
          .union([
            z.literal("sendrecv"),
            z.literal("sendonly"),
            z.literal("recvonly"),
            z.literal("inactive"),
          ])
          .optional(),
      })
    )
    .optional(),
});

const RtpParametersSchema = z.object({
  mid: z.string().optional(),
  codecs: z.array(
    z.object({
      mimeType: z.string(),
      payloadType: z.number(),
      clockRate: z.number(),
      channels: z.number().optional(),
      parameters: z.any().optional(),
      rtcpFeedback: z
        .array(
          z.object({
            type: z.string(),
            parameter: z.string().optional(),
          })
        )
        .optional(),
    })
  ),
  headerExtensions: z
    .array(
      z.object({
        uri: z.string(),
        id: z.number(),
        encrypt: z.boolean().optional(),
        parameters: z.any().optional(),
      })
    )
    .optional(),
  encodings: z
    .array(
      z.object({
        ssrc: z.number().optional(),
        rid: z.string().optional(),
        codecPayloadType: z.number().optional(),
        rtx: z
          .object({
            ssrc: z.number(),
          })
          .optional(),
        dtx: z.boolean().optional(),
        scalabilityMode: z.string().optional(),
        scaleResolutionDownBy: z.number().optional(),
        maxBitrate: z.number().optional(),
      })
    )
    .optional(),
  rtcp: z
    .object({
      cname: z.string().optional(),
      reducedSize: z.boolean().optional(),
      mux: z.boolean().optional(),
    })
    .optional(),
});

//JoinSpace
export const JoinSpaceDataSchema = z.object({
  spaceId: z.string(),
});
export type JoinSpaceData = z.infer<typeof JoinSpaceDataSchema>;
export const JoinSpaceResponseSchema = z.object({
  success: z.boolean(),
});
export type JoinSpaceResponse = z.infer<typeof JoinSpaceResponseSchema>;

//LeaveSpace
export const LeaveSpaceDataSchema = z.object({
  spaceId: z.string(),
});
export type LeaveSpaceData = z.infer<typeof LeaveSpaceDataSchema>;
export const LeaveSpaceResponseSchema = z.object({
  success: z.boolean(),
});
export type LeaveSpaceResponse = z.infer<typeof LeaveSpaceResponseSchema>;

//GetRouterRtpCapabilities
export const GetRouterRtpCapabilitiesResponseSchema = z.object({
  success: z.boolean(),
  routerRtpCapabilities: RtpCapabilitiesSchema.optional(),
});
export type GetRouterRtpCapabilitiesResponse = z.infer<
  typeof GetRouterRtpCapabilitiesResponseSchema
>;

//CreateTransport
export const CreateTransportResponseSchema = z.object({
  success: z.boolean(),
  params: z
    .object({
      id: z.string(),
      iceParameters: z.object({
        usernameFragment: z.string(),
        password: z.string(),
        iceLite: z.boolean().optional(),
      }),
      iceCandidates: z.array(
        z.object({
          foundation: z.string(),
          ip: z.string(),
          port: z.number(),
          priority: z.number(),
          protocol: z.union([z.literal("udp"), z.literal("tcp")]),
          type: z.literal("host"),
          tcpType: z.literal("passive").optional(),
        })
      ),
      dtlsParameters: z.object({
        role: z
          .union([z.literal("auto"), z.literal("client"), z.literal("server")])
          .optional(),
        fingerprints: z.array(
          z.object({
            algorithm: z.string(),
            value: z.string(),
          })
        ),
      }),
      sctpParameters: z
        .object({
          port: z.number(),
          OS: z.number(),
          MIS: z.number(),
          maxMessageSize: z.number(),
        })
        .optional(),
    })
    .optional(),
});
export type CreateTransportResponse = z.infer<
  typeof CreateTransportResponseSchema
>;

//NewAudioConsumer
export const NewAudioConsumerDataSchema = z.object({
  playerId: z.string(),
  producerId: z.string(),
  id: z.string(),
  kind: MediaTypeSchema,
  rtpParameters: RtpParametersSchema,
  type: z.union([
    z.literal("simulcast"),
    z.literal("svc"),
    z.literal("simple"),
    z.literal("pipe"),
  ]),
  producerPaused: z.boolean(),
});
export type NewAudioConsumerData = z.infer<typeof NewAudioConsumerDataSchema>;

//ConnectTransport
export const ConnectTransportDataSchema = z.object({
  dtlsParameters: z.object({
    role: z.union([
      z.literal("auto"),
      z.literal("client"),
      z.literal("server"),
    ]),
    fingerprints: z.array(
      z.object({
        algorithm: z.string(),
        value: z.string(),
      })
    ),
  }),
});
export type ConnectTransportData = z.infer<typeof ConnectTransportDataSchema>;
export const ConnectTransportResponseSchema = z.object({
  success: z.boolean(),
});
export type ConnectTransportResponse = z.infer<
  typeof ConnectTransportResponseSchema
>;

//ProduceAudio
export const ProduceAudioDataSchema = z.object({
  kind: MediaTypeSchema,
  rtpParameters: RtpParametersSchema,
});
export type ProduceAudioData = z.infer<typeof ProduceAudioDataSchema>;
export const ProduceAudioResponseSchema = z.object({
  success: z.boolean(),
  id: z.string().optional(),
});
export type ProduceAudioResponse = z.infer<typeof ProduceAudioResponseSchema>;

//ConsumeAudio
export const ConsumeAudioDataSchema = z.object({
  rtpCapabilities: RtpCapabilitiesSchema,
});
export type ConsumeAudioData = z.infer<typeof ConsumeAudioDataSchema>;
export const ConsumeAudioResposneSchema = z.object({
  success: z.boolean(),
});
export type ConsumeAudioResponse = z.infer<typeof ConsumeAudioResposneSchema>;

//ProduceData
export const ProduceDataDataSchema = z.object({
  sctpStreamParameters: z.object({
    streamId: z.number(),
    ordered: z.boolean().optional(),
    maxPacketLifeTime: z.number().optional(),
    maxRetransmits: z.number().optional(),
  }),
});
export type ProduceDataData = z.infer<typeof ProduceDataDataSchema>;
export const ProduceDataResponseSchema = z.object({
  success: z.boolean(),
  id: z.string().optional(),
});
export type ProduceDataResponse = z.infer<typeof ProduceDataResponseSchema>;

//ConsumeData
export const ConsumeDataDataSchema = z.object({
  sctpCapabilities: z.object({
    numStreams: z.object({
      OS: z.number(),
      MIS: z.number(),
    }),
  }),
});
export type ConsumeDataData = z.infer<typeof ConsumeDataDataSchema>;
export const ConsumeDataResponseSchema = z.object({
  success: z.boolean(),
});
export type ConsumeDataResponse = z.infer<typeof ConsumeDataResponseSchema>;

//NewDataConsumer
export const NewDataConsumerDataSchema = z.object({
  playerId: z.string(),
  dataProducerId: z.string(),
  id: z.string(),
  sctpStreamParameters: z
    .object({
      streamId: z.number(),
      ordered: z.boolean().optional(),
      maxPacketLifeTime: z.number().optional(),
      maxRetransmits: z.number().optional(),
    })
    .optional(),
});
export type NewDataConsumerData = z.infer<typeof NewDataConsumerDataSchema>;

//Identity
export const IdentityDataSchema = z.object({
  handle: z.union([z.string(), z.null()]),
});
export type IdentityData = z.infer<typeof IdentityDataSchema>;
export const IdentityResponseSchema = z.object({
  success: z.boolean(),
});
export type IdentityResponse = z.infer<typeof IdentityResponseSchema>;

//ChatMessage
export const SendChatMessageDataSchema = z.object({
  message: z.string(),
});
export type SendChatMessageData = z.infer<typeof SendChatMessageDataSchema>;
export const SendChatMessageResponseSchema = z.object({
  success: z.boolean(),
});
export type SendChatMessageResponse = z.infer<
  typeof SendChatMessageResponseSchema
>;

//RecieveChatMessage
export const ReceiveChatMessageDataSchema = z.object({
  messageId: z.string(),
  senderId: z.string(),
  senderName: z.string(),
  text: z.string(),
  timestamp: z.number(),
});
export type ReceiveChatMessageData = z.infer<
  typeof ReceiveChatMessageDataSchema
>;

//Location
export const LocationMessageSchema = z.object({
  type: z.literal("location"),
  position: z.number().array().length(3),
  rotation: z.number(),
});
export type LocationMessage = z.infer<typeof LocationMessageSchema>;

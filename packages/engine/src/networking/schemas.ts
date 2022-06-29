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
  headerExtensions: z.array(
    z.object({
      uri: z.string(),
      id: z.number(),
      encrypt: z.boolean().optional(),
      parameters: z.any().optional(),
    })
  ),
  encodings: z.array(
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
  ),
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
          type: z.union([
            z.literal("host"),
            z.literal("srflx"),
            z.literal("prflx"),
            z.literal("relay"),
          ]),
          tcpType: z
            .union([z.literal("active"), z.literal("passive"), z.literal("so")])
            .optional(),
        })
      ),
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
    })
    .optional(),
});
export type CreateTransportResponse = z.infer<
  typeof CreateTransportResponseSchema
>;

//Produce
export const ProduceResponseSchema = z.object({
  success: z.boolean(),
  id: z.string().optional(),
});

//NewConsumer
export const NewConsumerDataSchema = z.object({
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

//ConnectTransport
export const ConnectTransportSchema = z.object({
  success: z.boolean(),
});

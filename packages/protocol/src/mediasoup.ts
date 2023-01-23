import { z } from "zod";

export class MediasoupSchema {
  static dltsParameters = z.object({
    role: z.union([z.literal("auto"), z.literal("client"), z.literal("server")]).optional(),
    fingerprints: z.array(z.object({ algorithm: z.string(), value: z.string() })),
  });
  static rtcpFeedback = z.object({ type: z.string(), parameter: z.string().optional() });
  static rtpHeaderExtension = z.object({
    kind: z.union([z.literal("audio"), z.literal("video")]),
    uri: z.string(),
    preferredId: z.number().int().positive(),
    preferredEncrypt: z.boolean().optional(),
    direction: z
      .union([
        z.literal("sendrecv"),
        z.literal("sendonly"),
        z.literal("recvonly"),
        z.literal("inactive"),
      ])
      .optional(),
  });
  static rtpHeaderExtensionParameters = z.object({
    uri: z.string(),
    id: z.number().int().positive(),
    encrypt: z.boolean().optional(),
    parameters: z.record(z.string()).optional(),
  });
  static rtpParameters = z.object({
    mid: z.string().optional(),
    codecs: z.array(
      z.object({
        mimeType: z.string(),
        payloadType: z.number().int().positive(),
        clockRate: z.number().int().positive(),
        channels: z.number().int().positive().optional(),
        parameters: z.record(z.string()).optional(),
        rtcpFeedback: z.array(MediasoupSchema.rtcpFeedback).optional(),
      })
    ),
    headerExtensions: z.array(MediasoupSchema.rtpHeaderExtensionParameters).optional(),
    encodings: z
      .array(
        z.object({
          ssrc: z.number().int().positive().optional(),
          rid: z.string().optional(),
          codecPayloadType: z.number().int().positive().optional(),
          rtx: z.object({ ssrc: z.number().int().positive() }).optional(),
          dtx: z.boolean().optional(),
          scalabilityMode: z.string().optional(),
          scaleResolutionDownBy: z.number().optional(),
          maxBitrate: z.number().optional(),
          maxFramerate: z.number().optional(),
          adaptivePtime: z.boolean().optional(),
          priority: z
            .union([
              z.literal("very-low"),
              z.literal("low"),
              z.literal("medium"),
              z.literal("high"),
            ])
            .optional(),
          networkPriority: z
            .union([
              z.literal("very-low"),
              z.literal("low"),
              z.literal("medium"),
              z.literal("high"),
            ])
            .optional(),
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
  static rtpCapabilities = z.object({
    codecs: z
      .array(
        z.object({
          kind: z.union([z.literal("audio"), z.literal("video")]),
          mimeType: z.string(),
          preferredPayloadType: z.number().int().positive().optional(),
          clockRate: z.number().int().positive(),
          channels: z.number().int().positive().optional(),
          parameters: z.record(z.string()).optional(),
          rtcpFeedback: z.array(MediasoupSchema.rtcpFeedback).optional(),
        })
      )
      .optional(),
    headerExtensions: z.array(MediasoupSchema.rtpHeaderExtension).optional(),
  });
  static sctpStreamParameters = z.object({
    streamId: z.number().int().positive().optional(),
    ordered: z.boolean().optional(),
    maxPacketLifeTime: z.number().int().positive().optional(),
    maxRetransmits: z.number().int().positive().optional(),
    label: z.string().optional(),
    protocol: z.string().optional(),
  });
  static transportOptions = z.object({
    id: z.string(),
    iceParameters: z.object({
      usernameFragment: z.string(),
      password: z.string(),
      iceLite: z.boolean().optional(),
    }),
    iceCandidates: z.array(
      z.object({
        foundation: z.string(),
        priority: z.number().int().positive(),
        ip: z.string(),
        protocol: z.union([z.literal("udp"), z.literal("tcp")]),
        port: z.number().int().positive(),
        type: z.union([
          z.literal("host"),
          z.literal("srflx"),
          z.literal("prflx"),
          z.literal("relay"),
        ]),
        tcpType: z.union([z.literal("active"), z.literal("passive"), z.literal("so")]),
      })
    ),
    dtlsParameters: MediasoupSchema.dltsParameters,
    sctpParameters: z
      .object({
        port: z.number().int().positive(),
        OS: z.number().int().positive(),
        MIS: z.number().int().positive(),
        maxMessageSize: z.number().int().positive(),
      })
      .optional(),
    iceServers: z
      .array(
        z.object({
          credential: z.string().optional(),
          urls: z.union([z.string(), z.array(z.string())]),
          username: z.string().optional(),
        })
      )
      .optional(),
    iceTransportPolicy: z.union([z.literal("all"), z.literal("relay")]).optional(),
    additionalSettings: z.record(z.string()).optional(),
    proprietaryConstraints: z.record(z.string()).optional(),
    appData: z.record(z.string()).optional(),
  });
}

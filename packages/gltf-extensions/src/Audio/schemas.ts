import { z } from "zod";

const glTFid = z.number().min(0);

export const audioDataSchema = z
  .object({
    bufferView: glTFid.optional(),
    extensions: z.unknown().optional(),
    extras: z.unknown().optional(),
    mimeType: z.union([z.literal("audio/mpeg"), z.string()]).optional(),
    name: z.unknown().optional(),
    uri: z.string().optional(),
  })
  .refine(
    (obj) => {
      return (
        (obj.uri !== undefined && obj.bufferView === undefined) ||
        (obj.uri === undefined && obj.bufferView !== undefined)
      );
    },
    { message: 'Either "uri" or "bufferView" must be provided.' }
  )
  .refine(
    (obj) => {
      return !(obj.bufferView !== undefined && obj.mimeType === undefined);
    },
    { message: 'If "bufferView" is defined, "mimeType" must also be provided.' }
  );

export type AudioDataDef = z.infer<typeof audioDataSchema>;

export const audioSourceSchema = z.object({
  audio: glTFid,
  autoPlay: z.boolean().default(false),
  extensions: z.unknown().optional(),
  extras: z.unknown().optional(),
  gain: z.number().min(0).default(1.0),
  loop: z.boolean().default(false),
  name: z.string(),
});

export type AudioSourceDef = z.infer<typeof audioSourceSchema>;

const audioEmitterDistanceModelSchema = z.union([
  z.literal("linear"),
  z.literal("inverse"),
  z.literal("exponential"),
]);

export type AudioEmitterDistanceModel = z.infer<
  typeof audioEmitterDistanceModelSchema
>;

const positionalEmitterSchema = z.object({
  coneInnerAngle: z
    .number()
    .min(0.0)
    .max(6.283185307179586)
    .default(6.283185307179586),
  coneOuterAngle: z
    .number()
    .min(0.0)
    .max(6.283185307179586)
    .default(6.283185307179586),
  coneOuterGain: z.number().min(0.0).max(1.0).default(0.0),
  distanceModel: audioEmitterDistanceModelSchema.default("inverse"),
  extensions: z.unknown().optional(),
  extras: z.unknown().optional(),
  maxDistance: z.number().gt(0.0).default(10000.0),
  refDistance: z.number().min(0.0).default(1.0),
  rolloffFactor: z.number().min(0.0).default(1.0),
});

export type AudioEmitterPositionalDef = Partial<
  z.infer<typeof positionalEmitterSchema>
>;

const audioEmitterTypeSchema = z.union([
  z.literal("positional"),
  z.literal("global"),
]);

export type AudioEmitterType = z.infer<typeof audioEmitterTypeSchema>;

export const audioEmitterSchema = z
  .object({
    extensions: z.unknown().optional(),
    extras: z.unknown().optional(),
    gain: z.number().min(0.0).default(1.0),
    name: z.string(),
    positional: positionalEmitterSchema.partial().optional(),
    sources: z.array(glTFid),
    type: audioEmitterTypeSchema,
  })
  .refine(
    (obj) => {
      return !(obj.type === "positional" && !obj.positional);
    },
    {
      message:
        "Positional audio emitter must have 'positional' property defined",
    }
  );

export type AudioEmitterDef = z.infer<typeof audioEmitterSchema>;

export const sceneAudioSchema = z.object({
  emitters: z.array(glTFid).min(1),
  extensions: z.unknown().optional(),
  extras: z.unknown().optional(),
});

export type SceneAudioDef = z.infer<typeof sceneAudioSchema>;

export const nodeAudioSchema = z.object({
  emitter: glTFid,
  extensions: z.unknown().optional(),
  extras: z.unknown().optional(),
});

export type NodeAudioDef = z.infer<typeof nodeAudioSchema>;

export const audioExtensionSchema = z.object({
  audio: z.array(audioDataSchema).min(1),
  emitters: z.array(audioEmitterSchema).min(1),
  extensions: z.unknown().optional(),
  extras: z.unknown().optional(),
  sources: z.array(audioSourceSchema).min(1),
});

export type AudioExtensionDef = z.infer<typeof audioExtensionSchema>;

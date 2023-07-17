import { z } from "zod";

const colliderTypeSchema = z.enum([
  "box",
  "sphere",
  "capsule",
  "cylinder",
  "hull",
  "trimesh",
]);

export type ColliderType = z.infer<typeof colliderTypeSchema>;

const glTFid = z.number().min(0);

export const colliderSchema = z
  .object({
    height: z.number().optional(),
    isTrigger: z.boolean().default(false),
    mesh: glTFid.optional(),
    radius: z.number().optional(),
    size: z.array(z.number()).length(3).optional(),
    type: colliderTypeSchema,
  })
  .refine((obj) => {
    try {
      switch (obj.type) {
        case "box":
          z.object({
            size: z.array(z.number()).length(3),
            type: z.literal("box"),
          }).parse(obj);
          break;
        case "sphere":
          z.object({
            radius: z.number(),
            type: z.literal("sphere"),
          }).parse(obj);
          break;
        case "capsule":
          z.object({
            height: z.number(),
            radius: z.number(),
            type: z.literal("capsule"),
          }).parse(obj);
          break;
        case "cylinder":
          z.object({
            height: z.number(),
            radius: z.number(),
            type: z.literal("cylinder"),
          }).parse(obj);
          break;
        case "hull":
          z.object({
            mesh: glTFid,
            type: z.literal("hull"),
          }).parse(obj);
          break;
        case "trimesh":
          z.object({
            mesh: glTFid,
            type: z.literal("trimesh"),
          }).parse(obj);
          break;
        default:
          return false;
      }
      return true;
    } catch (error) {
      return false;
    }
  });

export type ColliderDef = z.infer<typeof colliderSchema>;

export const colliderExtensionSchema = z.object({
  colliders: z.array(colliderSchema).min(1),
  extensions: z.unknown().optional(),
  extras: z.unknown().optional(),
});

export type ColliderExtensionDef = z.infer<typeof colliderExtensionSchema>;

export const nodeColliderSchema = z.object({
  collider: glTFid,
  extensions: z.unknown().optional(),
  extras: z.unknown().optional(),
});

export type NodeColliderDef = z.infer<typeof nodeColliderSchema>;

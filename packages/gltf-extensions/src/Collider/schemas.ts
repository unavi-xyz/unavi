import { z } from "zod";

const colliderTypeSchema = z.enum(["box", "sphere", "capsule", "cylinder", "hull", "trimesh"]);

export type ColliderType = z.infer<typeof colliderTypeSchema>;

const glTFid = z.number().min(0);

export const colliderSchema = z
  .object({
    type: colliderTypeSchema,
    isTrigger: z.boolean().default(false),
    size: z.array(z.number()).length(3).optional(),
    radius: z.number().optional(),
    height: z.number().optional(),
    mesh: glTFid.optional(),
  })
  .refine((obj) => {
    try {
      switch (obj.type) {
        case "box":
          z.object({
            type: z.literal("box"),
            size: z.array(z.number()).length(3),
          }).parse(obj);
          break;
        case "sphere":
          z.object({
            type: z.literal("sphere"),
            radius: z.number(),
          }).parse(obj);
          break;
        case "capsule":
          z.object({
            type: z.literal("capsule"),
            radius: z.number(),
            height: z.number(),
          }).parse(obj);
          break;
        case "cylinder":
          z.object({
            type: z.literal("cylinder"),
            radius: z.number(),
            height: z.number(),
          }).parse(obj);
          break;
        case "hull":
          z.object({
            type: z.literal("hull"),
            mesh: glTFid,
          }).parse(obj);
          break;
        case "trimesh":
          z.object({
            type: z.literal("trimesh"),
            mesh: glTFid,
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

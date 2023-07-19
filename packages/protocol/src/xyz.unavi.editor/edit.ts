import { z } from "zod";

export const EditNodeSchema = z.object({
  data: z.object({
    mesh: z.string().nullable().optional(),
    name: z.string().optional(),
    parent: z.string().nullable().optional(),
    target: z.string(),
  }),
  id: z.literal("xyz.unavi.editor.edit.node"),
  target: z.literal("client"),
});
export type EditNode = z.infer<typeof EditNodeSchema>;

export const EditMeshSchema = z.object({
  data: z.object({
    colors: z.array(z.number()).nullable().optional(),
    indices: z.array(z.number()).nullable().optional(),
    joints: z.array(z.number()).nullable().optional(),
    material: z.string().nullable().optional(),
    name: z.string().nullable().optional(),
    normals: z.array(z.number()).nullable().optional(),
    positions: z.array(z.number()).nullable().optional(),
    target: z.string(),
    uv: z.array(z.number()).nullable().optional(),
    uv1: z.array(z.number()).nullable().optional(),
    uv2: z.array(z.number()).nullable().optional(),
    uv3: z.array(z.number()).nullable().optional(),
    weights: z.array(z.number()).nullable().optional(),
  }),
  id: z.literal("xyz.unavi.editor.edit.mesh"),
  target: z.literal("client"),
});
export type EditMesh = z.infer<typeof EditMeshSchema>;

export const EditMessageSchema = z.union([EditNodeSchema, EditMeshSchema]);
export type EditMessage = z.infer<typeof EditMessageSchema>;

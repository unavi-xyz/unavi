import { z } from "zod";

export const AddNodeSchema = z.object({
  data: z.object({
    mesh: z.string().optional(),
    name: z.string(),
    parent: z.string().optional(),
  }),
  id: z.literal("xyz.unavi.editor.add.node"),
  target: z.literal("client"),
});

export type AddNode = z.infer<typeof AddNodeSchema>;

export const AddMeshSchema = z.object({
  data: z.object({
    colors: z.array(z.number()).optional(),
    indices: z.array(z.number()).optional(),
    joints: z.array(z.number()).optional(),
    material: z.string().optional(),
    name: z.string(),
    normals: z.array(z.number()).optional(),
    positions: z.array(z.number()).optional(),
    uv: z.array(z.number()).optional(),
    uv1: z.array(z.number()).optional(),
    uv2: z.array(z.number()).optional(),
    uv3: z.array(z.number()).optional(),
    weights: z.array(z.number()).optional(),
  }),
  id: z.literal("xyz.unavi.editor.add.mesh"),
  target: z.literal("client"),
});

export type AddMesh = z.infer<typeof AddMeshSchema>;

export const AddMessageSchema = z.union([AddNodeSchema, AddMeshSchema]);

export type AddMessage = z.infer<typeof AddMessageSchema>;

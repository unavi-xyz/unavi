import { z } from "zod";

export const AddNodeSchema = z.object({
  data: z.object({
    name: z.string(),
  }),
  id: z.literal("xyz.unavi.editor.add.node"),
  target: z.literal("client"),
});
export type AddNode = z.infer<typeof AddNodeSchema>;

export const AddMeshSchema = z.object({
  data: z.object({
    name: z.string(),
  }),
  id: z.literal("xyz.unavi.editor.add.mesh"),
  target: z.literal("client"),
});
export type AddMesh = z.infer<typeof AddMeshSchema>;

export const AddMessageSchema = z.union([AddNodeSchema, AddMeshSchema]);
export type AddMessage = z.infer<typeof AddMessageSchema>;

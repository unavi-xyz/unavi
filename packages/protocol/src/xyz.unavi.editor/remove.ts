import { z } from "zod";

export const RemoveNodeSchema = z.object({
  data: z.object({
    name: z.string().optional(),
    parent: z.string().optional(),
  }),
  id: z.literal("xyz.unavi.editor.remove.node"),
  target: z.literal("client"),
});

export type RemoveNode = z.infer<typeof RemoveNodeSchema>;

export const RemoveMeshSchema = z.object({
  data: z.object({
    name: z.string().optional(),
  }),
  id: z.literal("xyz.unavi.editor.remove.mesh"),
  target: z.literal("client"),
});

export type RemoveMesh = z.infer<typeof RemoveMeshSchema>;

export const RemoveMessageSchema = z.union([
  RemoveNodeSchema,
  RemoveMeshSchema,
]);

export type RemoveMessage = z.infer<typeof RemoveMessageSchema>;

import z from "zod";

export type PublishProjectResponse = { id: string; modelSize: number };

export const postSchema = z.object({ optimize: z.boolean() });
export type PostSchema = z.infer<typeof postSchema>;

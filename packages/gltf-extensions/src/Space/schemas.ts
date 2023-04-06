import { z } from "zod";

export const spaceSchema = z.object({
  host: z.string().url(),
  image: z.string().optional(),
});

export type SpaceDef = z.infer<typeof spaceSchema>;

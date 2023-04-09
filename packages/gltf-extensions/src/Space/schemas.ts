import { z } from "zod";

export const spaceSchema = z.object({
  host: z.string(),
  image: z.string().optional(),
});

export type SpaceDef = z.infer<typeof spaceSchema>;

import { z } from "zod";

export const putSchema = z.object({
  spaceId: z.number().int().positive(),
});

export type PutParams = z.infer<typeof putSchema>;

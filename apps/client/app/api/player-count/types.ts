import { z } from "zod";

export const postPlayerCountSchema = z.object({
  host: z.string(),
  uri: z.string(),
});

export type PostPlayerCountArgs = z.infer<typeof postPlayerCountSchema>;

export type PostPlayerCountResponse = {
  count: number;
};

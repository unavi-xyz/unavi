import { z } from "zod";

export const spawnPointSchema = z.object({
  group: z.string().max(128),
  team: z.string().max(128),
  title: z.string().max(128),
});

export type SpawnPointDef = z.infer<typeof spawnPointSchema>;

import { z } from "zod";

export const spawnPointSchema = z.object({
  title: z.string().max(128),
  team: z.string().max(128),
  group: z.string().max(128),
});

export type SpawnPointDef = z.infer<typeof spawnPointSchema>;

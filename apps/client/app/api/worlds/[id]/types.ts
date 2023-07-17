import { z } from "zod";

import {
  WORLD_DESCRIPTION_LENGTH,
  WORLD_ID_LENGTH,
  WORLD_TITLE_LENGTH,
} from "@/src/server/db/constants";

export type Params = { params: { id: string } };

export const paramsSchema = z.object({
  id: z.string().length(WORLD_ID_LENGTH),
});

export const patchBodySchema = z.object({
  description: z.string().max(WORLD_DESCRIPTION_LENGTH).optional(),
  title: z.string().max(WORLD_TITLE_LENGTH).optional(),
});

export type PatchBody = z.infer<typeof patchBodySchema>;

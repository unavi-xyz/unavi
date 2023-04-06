import { z } from "zod";

import {
  MAX_DESCRIPTION_LENGTH,
  MAX_TITLE_LENGTH,
  PROJECT_ID_LENGTH,
  PUBLICATION_ID_LENGTH,
} from "../constants";

export type Params = { params: { id: string } };

export const paramsSchema = z.object({
  id: z.string().length(PROJECT_ID_LENGTH),
});

export const patchSchema = z.object({
  title: z.string().max(MAX_TITLE_LENGTH).optional(),
  description: z.string().max(MAX_DESCRIPTION_LENGTH).optional(),
  publicationId: z.string().length(PUBLICATION_ID_LENGTH).nullable().optional(),
});

export type PatchSchema = z.infer<typeof patchSchema>;

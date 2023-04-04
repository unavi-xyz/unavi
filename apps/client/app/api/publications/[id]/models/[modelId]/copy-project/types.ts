import { z } from "zod";

import {
  PROJECT_ID_LENGTH,
  PUBLICATION_ID_LENGTH,
  PUBLISHED_MODEL_ID_LENGTH,
} from "@/app/api/projects/constants";

export const paramsSchema = z.object({
  id: z.string().length(PUBLICATION_ID_LENGTH),
  modelId: z.string().length(PUBLISHED_MODEL_ID_LENGTH),
});

export const postSchema = z.object({
  projectId: z.string().length(PROJECT_ID_LENGTH),
});

export type PostParams = z.infer<typeof postSchema>;

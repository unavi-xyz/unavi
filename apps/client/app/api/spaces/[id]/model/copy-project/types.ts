import { z } from "zod";

import { PROJECT_ID_LENGTH } from "@/app/api/projects/constants";

export const postSchema = z.object({
  projectId: z.string().length(PROJECT_ID_LENGTH),
});

export type PostParams = z.infer<typeof postSchema>;

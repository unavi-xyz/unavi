import { z } from "zod";

import { PUBLICATION_ID_LENGTH } from "../../../../../../projects/constants";
import { PUBLISHED_MODEL_ID_LENGTH } from "../../../../../../projects/constants";
import { PUBLISHED_MODEL_FILE } from "../../files";

export type Params = { params: { id: string; file: string } };

export const paramsSchema = z.object({
  id: z.string().length(PUBLICATION_ID_LENGTH),
  modelId: z.string().length(PUBLISHED_MODEL_ID_LENGTH),
  file: z.union([z.literal(PUBLISHED_MODEL_FILE.IMAGE), z.literal(PUBLISHED_MODEL_FILE.MODEL)]),
});

export type GetFileUploadResponse = { url: string };

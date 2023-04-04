import { z } from "zod";

import { PUBLICATION_ID_LENGTH } from "../../../../projects/constants";
import { PUBLICATION_FILE } from "../../files";

export type Params = { params: { id: string; file: string } };

export const paramsSchema = z.object({
  id: z.string().length(PUBLICATION_ID_LENGTH),
  file: z.literal(PUBLICATION_FILE.METADATA),
});

export type GetFileUploadResponse = { url: string };

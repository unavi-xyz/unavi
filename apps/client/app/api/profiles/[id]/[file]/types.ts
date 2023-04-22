import { z } from "zod";

import { PROFILE_FILE } from "../../files";

export type Params = { params: { id: string; file: string } };

export const paramsSchema = z.object({
  id: z.string(),
  file: z.union([
    z.literal(PROFILE_FILE.BACKGROUND),
    z.literal(PROFILE_FILE.IMAGE),
    z.literal(PROFILE_FILE.METADATA),
  ]),
});

export type GetFileUploadResponse = { url: string };

import { z } from "zod";

import { SPACE_ID_LENGTH } from "@/app/api/projects/constants";

import { SPACE_MODEL_FILE } from "./files";

export type Params = { params: { id: string; file: string } };

export const paramsSchema = z.object({
  id: z.string().length(SPACE_ID_LENGTH),
  file: z.union([z.literal(SPACE_MODEL_FILE.IMAGE), z.literal(SPACE_MODEL_FILE.MODEL)]),
});

export type GetFileDownloadResponse = { url: string };
export type GetFileUploadResponse = { url: string };

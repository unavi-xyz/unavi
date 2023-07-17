import { z } from "zod";

import { WORLD_ID_LENGTH } from "@/src/server/db/constants";

import { WORLD_MODEL_FILE } from "./files";

export type Params = { params: { id: string; file: string } };

export const paramsSchema = z.object({
  file: z.union([
    z.literal(WORLD_MODEL_FILE.IMAGE),
    z.literal(WORLD_MODEL_FILE.MODEL),
  ]),
  id: z.string().length(WORLD_ID_LENGTH),
});

export type GetFileDownloadResponse = { url: string };
export type GetFileUploadResponse = { url: string };

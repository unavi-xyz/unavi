import { z } from "zod";

import { PROJECT_ID_LENGTH } from "../../../constants";
import { PROJECT_FILE } from "./files";

export type Params = { params: { id: string; file: string } };

export const paramsSchema = z.object({
  file: z.union([z.literal(PROJECT_FILE.IMAGE), z.literal(PROJECT_FILE.MODEL)]),
  id: z.string().length(PROJECT_ID_LENGTH),
});

export type GetFileDownloadResponse = { url: string };
export type GetFileUploadResponse = { url: string };

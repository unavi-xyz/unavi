import { z } from "zod";

import { SPACE_ID_LENGTH } from "@/app/api/projects/constants";

import { SPACE_NFT_FILE } from "./files";

export type Params = { params: { id: string; file: string } };

export const paramsSchema = z.object({
  id: z.string().length(SPACE_ID_LENGTH),
  file: z.literal(SPACE_NFT_FILE.metadata),
});

export type GetFileDownloadResponse = { url: string };
export type GetFileUploadResponse = { url: string };

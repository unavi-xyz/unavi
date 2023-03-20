import { z } from "zod";

import { PROJECT_ID_LENGTH } from "../../constants";

export type Params = { params: { id: string } };

export const paramsSchema = z.object({
  id: z.string().length(PROJECT_ID_LENGTH),
});

export type PostAssetsResponse = { url: string; assetId: string };

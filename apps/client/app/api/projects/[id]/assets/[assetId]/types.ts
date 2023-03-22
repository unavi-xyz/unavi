import { z } from "zod";

import { ASSET_ID_LENGTH, PROJECT_ID_LENGTH } from "../../../constants";

export type Params = { params: { id: string } };

export const paramsSchema = z.object({
  id: z.string().length(PROJECT_ID_LENGTH),
  assetId: z.string().length(ASSET_ID_LENGTH),
});

export type PutAssetResponse = { url: string };

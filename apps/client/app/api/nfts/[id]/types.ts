import { z } from "zod";

import { SPACE_NFT_ID_LENGTH } from "../../projects/constants";

export type Params = { params: { id: string } };

export const paramsSchema = z.object({
  id: z.string().length(SPACE_NFT_ID_LENGTH),
});

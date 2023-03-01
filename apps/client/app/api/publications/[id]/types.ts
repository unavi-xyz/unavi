import { z } from "zod";

import { PUBLICATION_ID_LENGTH } from "../../projects/constants";

export type Params = { params: { id: string } };
export const paramsSchema = z.object({
  id: z.string().length(PUBLICATION_ID_LENGTH),
});

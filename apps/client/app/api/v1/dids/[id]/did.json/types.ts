import { z } from "zod";

import { DID_ID_LENGTH } from "@/src/server/db/constants";

export const paramsSchema = z.object({
  id: z.string().length(DID_ID_LENGTH),
});

export type Params = { params: z.infer<typeof paramsSchema> };

import { z } from "zod";

import { fetchSpace } from "../../../../src/server/helpers/fetchSpace";

export const paramsSchema = z.object({
  id: z.string().max(100),
});

export type GetSpaceResponse = Awaited<ReturnType<typeof fetchSpace>>;

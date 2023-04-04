import { z } from "zod";

import { fetchSpace } from "@/src/server/helpers/fetchSpace";

export type Params = { params: { id: string } };

export const paramsSchema = z.object({
  id: z.string().max(100),
});

export type GetSpaceResponse = Awaited<ReturnType<typeof fetchSpace>>;

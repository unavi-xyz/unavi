import { z } from "zod";

import { SPACE_ID_LENGTH } from "../constants";

export type Params = { params: { id: string } };

export const paramsSchema = z.object({
  id: z.string().length(SPACE_ID_LENGTH),
});

export type GetResponse = {
  ownerId: string;
  uri: string | null;
};

import { z } from "zod";

import { SPACE_ID_LENGTH } from "../../constants";

export const putSchema = z.object({
  spaceId: z.string().length(SPACE_ID_LENGTH),
});

export type PutParams = z.infer<typeof putSchema>;

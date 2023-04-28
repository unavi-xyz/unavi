import { z } from "zod";

import { SPACE_ID_LENGTH } from "../../projects/constants";

export type Params = { params: { id: string } };

export const paramsSchema = z.object({
  id: z.string().length(SPACE_ID_LENGTH),
});

export const patchSchema = z.object({
  tokenId: z.number().int().min(0),
});

export type PatchSpaceRequest = z.infer<typeof patchSchema>;

export type GetSpaceResponse = {
  ownerId: string;
  uri: string | null;
};

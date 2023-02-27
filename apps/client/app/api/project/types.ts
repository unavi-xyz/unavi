import { z } from "zod";

import { MAX_NAME_LENGTH } from "./constants";

export const schema = z.object({ name: z.string().max(MAX_NAME_LENGTH).optional() });
export type CreateProjectArgs = z.infer<typeof schema>;
export type CreateProjectResponse = { id: string };

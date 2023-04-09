import { z } from "zod";

import { MAX_TITLE_LENGTH } from "./constants";

export const schema = z.object({ title: z.string().max(MAX_TITLE_LENGTH).optional() });

export type CreateProjectArgs = z.infer<typeof schema>;
export type CreateProjectResponse = { id: string };

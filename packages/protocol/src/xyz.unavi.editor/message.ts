import { z } from "zod";

import { AddMessageSchema } from "./add";
import { RemoveMessageSchema } from "./remove";

export const EditorMessageSchema = z.union([
  AddMessageSchema,
  RemoveMessageSchema,
]);

export type EditorMessage = z.infer<typeof EditorMessageSchema>;

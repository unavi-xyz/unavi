import { z } from "zod";

import { AddMessageSchema } from "./add";
import { EditMessageSchema } from "./edit";
import { RemoveMessageSchema } from "./remove";

export const EditorMessageSchema = z.union([
  AddMessageSchema,
  EditMessageSchema,
  RemoveMessageSchema,
]);

export type EditorMessage = z.infer<typeof EditorMessageSchema>;

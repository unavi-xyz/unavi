import { z } from "zod";

export const avatarSchema = z.object({
  name: z.string().optional(),
  equippable: z.boolean().optional(),
  uri: z.string(),
});

export type AvatarDef = z.infer<typeof avatarSchema>;

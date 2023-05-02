import { z } from "zod";

export const avatarSchema = z.object({
  equippable: z.boolean().optional(),
  name: z.string().optional(),
  uri: z.string(),
});

export type AvatarDef = z.infer<typeof avatarSchema>;

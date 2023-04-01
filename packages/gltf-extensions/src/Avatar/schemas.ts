import { z } from "zod";

export const avatarSchema = z.object({
  name: z.string(),
  equippable: z.boolean(),
  uri: z.string(),
});

export type AvatarDef = z.infer<typeof avatarSchema>;

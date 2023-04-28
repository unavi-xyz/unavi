import { z } from "zod";

import { MAX_PROFILE_NAME_LENGTH, MAX_USERNAME_LENGTH, MIN_USERNAME_LENGTH } from "./constants";

export const UpdateProfileSchema = z.object({
  username: z.string().min(MIN_USERNAME_LENGTH).max(MAX_USERNAME_LENGTH).optional(),
  name: z.string().max(MAX_PROFILE_NAME_LENGTH).optional(),
  bio: z.string().max(MAX_PROFILE_NAME_LENGTH).optional(),
  image: z.string().url().optional(),
  background: z.string().url().optional(),
});

export type UpdateProfileInput = z.infer<typeof UpdateProfileSchema>;

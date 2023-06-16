import { z } from "zod";

import {
  MAX_PROFILE_NAME_LENGTH,
  MAX_USERNAME_LENGTH,
  MIN_USERNAME_LENGTH,
} from "./constants";

export const UpdateProfileSchema = z.object({
  background: z.string().url().optional(),
  bio: z.string().max(MAX_PROFILE_NAME_LENGTH).optional(),
  image: z.string().url().optional(),
  name: z.string().max(MAX_PROFILE_NAME_LENGTH).optional(),
  username: z
    .string()
    .min(MIN_USERNAME_LENGTH)
    .max(MAX_USERNAME_LENGTH)
    .optional(),
});

export type UpdateProfileInput = z.infer<typeof UpdateProfileSchema>;

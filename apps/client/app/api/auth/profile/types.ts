import { z } from "zod";

import {
  FILE_KEY_LENGTH,
  MAX_PROFILE_NAME_LENGTH,
  MAX_USERNAME_LENGTH,
  MIN_USERNAME_LENGTH,
} from "@/src/server/db/constants";

export const UpdateProfileSchema = z.object({
  backgroundKey: z.string().length(FILE_KEY_LENGTH).optional(),
  bio: z.string().max(MAX_PROFILE_NAME_LENGTH).optional(),
  imageKey: z.string().length(FILE_KEY_LENGTH).optional(),
  name: z.string().max(MAX_PROFILE_NAME_LENGTH).optional(),
  username: z
    .string()
    .min(MIN_USERNAME_LENGTH)
    .max(MAX_USERNAME_LENGTH)
    .optional(),
});

export type UpdateProfileInput = z.infer<typeof UpdateProfileSchema>;

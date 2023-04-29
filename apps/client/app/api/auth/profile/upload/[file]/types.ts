import { z } from "zod";

export enum ProfileFile {
  image = "image",
  background = "background",
}

export const paramsSchema = z.object({
  file: z.nativeEnum(ProfileFile),
});

export type GetFileUploadResponse = { url: string; fileId: string };

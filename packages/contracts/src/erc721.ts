import { z } from "zod";

export const ERC721MetadataSchema = z.object({
  animation_url: z.string(),
  description: z.string(),
  external_url: z.string(),
  image: z.string(),
  name: z.string(),
});

export type ERC721Metadata = z.infer<typeof ERC721MetadataSchema>;

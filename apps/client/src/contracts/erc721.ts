import { z } from "zod";

export const ERC721MetadataSchema = z.object({
  animation_url: z.string().url().optional(),
  attributes: z
    .array(
      z.object({
        display_type: z.string().optional(),
        trait_type: z.string(),
        value: z.string(),
      })
    )
    .optional(),
  background_color: z.string().optional(),
  description: z.string().optional(),
  external_url: z.string().url().optional(),
  image: z.string().url().optional(),
  name: z.string().optional(),
});

export type ERC721Metadata = z.infer<typeof ERC721MetadataSchema>;

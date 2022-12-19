import { z } from "zod";

export interface ERC721Metadata {
  animation_url: string;
  description: string;
  external_url: string;
  image: string;
  name: string;
}

export const ERC721MetadataSchema = z.object({
  animation_url: z.string(),
  description: z.string(),
  external_url: z.string(),
  image: z.string(),
  name: z.string(),
});

import { z } from "zod";

import { fetchSpaceMetadata } from "./fetchSpaceMetadata";

export async function processSpaceURI(uri: string) {
  const isHttps = httpsSchema.safeParse(uri).success;
  const isNft = nftSchema.safeParse(uri).success;

  if (isHttps) {
    return uri;
  }

  if (isNft) {
    try {
      // Fetch metadata
      const tokenId = parseInt(uri.slice(6));
      const metadata = await fetchSpaceMetadata(tokenId);

      // No model
      if (!metadata?.animation_url) return null;

      return metadata.animation_url;
    } catch {
      return null;
    }
  }

  return null;
}

export const httpsSchema = z.string().refine((param) => param.startsWith("https://"));

export const nftSchema = z.string().refine((param) => {
  if (!param.startsWith("nft://")) return;

  const id = param.slice(6);
  const int = parseInt(id);

  return !isNaN(int) && int > 0;
});

export const idSchema = z.string().refine((param) => {
  if (!param.startsWith("id://")) return false;

  const id = param.slice(5);
  const int = parseInt(id);

  return !isNaN(int) && int > 0;
});

import { z } from "zod";

import { SPACE_ID_LENGTH } from "@/app/api/projects/constants";

import { fetchSpaceNFTMetadata } from "./fetchSpaceNFTMetadata";

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
      const metadata = await fetchSpaceNFTMetadata(tokenId);

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

export const idSchema = z.string().refine((param) => {
  return !param.startsWith("0x") && param.length === SPACE_ID_LENGTH;
});

export const nftSchema = z.string().refine((param) => {
  return param.startsWith("0x");
});

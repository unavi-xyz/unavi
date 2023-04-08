import { cache } from "react";

import { fetchNFTSpaceTokenMetadata } from "./fetchNFTSpaceTokenMetadata";
import { readSpaceMetadata } from "./readSpaceMetadata";

export const fetchNFTSpaceMetadata = cache(async (id: number) => {
  try {
    const erc721metadata = await fetchNFTSpaceTokenMetadata(id);
    if (!erc721metadata?.animation_url) return null;

    const metadata = await readSpaceMetadata(erc721metadata.animation_url);
    if (!metadata) return null;

    return metadata;
  } catch {
    return null;
  }
});

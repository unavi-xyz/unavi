import { cache } from "react";

import { fetchNFTSpaceTokenMetadata } from "./fetchNFTSpaceTokenMetadata";
import { fetchWorldMetadata } from "./fetchWorldMetadata";

export const fetchNFTSpaceMetadata = cache(async (id: number) => {
  try {
    const erc721metadata = await fetchNFTSpaceTokenMetadata(id);
    if (!erc721metadata?.animation_url) return null;

    const metadata = await fetchWorldMetadata(erc721metadata.animation_url);

    return metadata;
  } catch {
    return null;
  }
});

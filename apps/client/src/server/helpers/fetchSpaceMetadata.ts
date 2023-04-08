import { SpaceId } from "@/src/utils/parseSpaceId";

import { fetchSpaceDBMetadata } from "./fetchSpaceDBMetadata";
import { fetchSpaceNFTMetadata } from "./fetchSpaceNFTMetadata";
import { readSpaceMetadata, SpaceMetadata } from "./readSpaceMetadata";

export async function fetchSpaceMetadata(id: SpaceId): Promise<SpaceMetadata | null> {
  if (id.type === "tokenId") {
    const erc721metadata = await fetchSpaceNFTMetadata(id.value);
    if (!erc721metadata?.animation_url) return null;

    const metadata = await readSpaceMetadata(erc721metadata.animation_url);
    if (!metadata) null;

    return metadata;
  } else {
    return fetchSpaceDBMetadata(id.value);
  }
}

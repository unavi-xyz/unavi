import { SpaceId } from "@/src/utils/parseSpaceId";

import { DBSpaceMetadata, fetchDBSpaceMetadata } from "./fetchDBSpaceMetadata";
import { fetchNFTSpaceMetadata } from "./fetchNFTSpaceMetadata";
import { SpaceMetadata } from "./readSpaceMetadata";

export function fetchSpaceMetadata(id: SpaceId): Promise<SpaceMetadata | DBSpaceMetadata | null> {
  if (id.type === "tokenId") {
    return fetchNFTSpaceMetadata(id.value);
  } else {
    return fetchDBSpaceMetadata(id.value);
  }
}

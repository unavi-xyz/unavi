import { SpaceId } from "@/src/utils/parseSpaceId";

import { fetchDBSpaceMetadata } from "./fetchDBSpaceMetadata";
import { fetchNFTSpaceMetadata } from "./fetchNFTSpaceMetadata";

export function fetchSpaceMetadata(id: SpaceId) {
  if (id.type === "tokenId") {
    return fetchNFTSpaceMetadata(id.value);
  } else {
    return fetchDBSpaceMetadata(id.value);
  }
}

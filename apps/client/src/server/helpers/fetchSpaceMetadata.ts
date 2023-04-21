import { WorldMetadata } from "@wired-protocol/types";

import { SpaceId } from "@/src/utils/parseSpaceId";

import { DBSpaceMetadata, fetchDBSpaceMetadata } from "./fetchDBSpaceMetadata";
import { fetchNFTSpaceMetadata } from "./fetchNFTSpaceMetadata";

export function fetchSpaceMetadata(id: SpaceId): Promise<DBSpaceMetadata | WorldMetadata | null> {
  if (id.type === "tokenId") {
    return fetchNFTSpaceMetadata(id.value);
  } else {
    return fetchDBSpaceMetadata(id.value);
  }
}

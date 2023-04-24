import { SpaceId } from "@/src/utils/parseSpaceId";

import { fetchDBSpaceURI } from "./fetchDBSpaceURI";
import { fetchNFTSpaceURI } from "./fetchNFTSpaceURI";
import { fetchWorldMetadata } from "./fetchWorldMetadata";

export async function fetchSpaceMetadata(id: SpaceId) {
  let uri: string | null = null;

  if (id.type === "tokenId") {
    uri = await fetchNFTSpaceURI(id.value);
  } else if (id.type === "id") {
    const res = await fetchDBSpaceURI(id.value);
    if (res) uri = res.uri;
  } else if (id.type === "uri") {
    uri = id.value;
  }

  if (!uri) return null;

  return fetchWorldMetadata(uri);
}

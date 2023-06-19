import { cache } from "react";

import { SpaceId } from "@/src/utils/parseSpaceId";

import { fetchWorldMetadata } from "./fetchWorldMetadata";
import { fetchWorldURI } from "./fetchWorldURI";

export const fetchWorld = cache(async (id: SpaceId) => {
  let uri: string | null = null;

  if (id.type === "id") {
    const res = await fetchWorldURI(id.value);
    if (res) uri = res.uri;
  } else if (id.type === "uri") {
    uri = id.value;
  }

  if (!uri) return null;

  return await fetchWorldMetadata(uri);
});

import { WorldMetadataSchema } from "@wired-protocol/types";

import { SpaceId } from "@/src/utils/parseSpaceId";

import { fetchDBSpaceURI } from "./fetchDBSpaceURI";

export async function fetchWorldMetadata(id: SpaceId) {
  let uri: string | null = null;

  if (id.type === "id") {
    const res = await fetchDBSpaceURI(id.value);
    if (res) uri = res.uri;
  } else if (id.type === "uri") {
    uri = id.value;
  }

  if (!uri) return null;

  try {
    const res = await fetch(uri);
    const json = await res.json();

    const metadata = WorldMetadataSchema.parse(json);

    return { metadata, uri };
  } catch {
    return null;
  }
}

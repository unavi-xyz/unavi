import { cache } from "react";

import { env } from "@/src/env.mjs";
import { WorldId } from "@/src/utils/parseWorldId";

import { fetchWorldMetadata } from "./fetchWorldMetadata";
import { getWorldJson } from "./getWorldJson";

export const fetchWorld = cache(async (id: WorldId) => {
  let uri: string | null = null;

  if (id.type === "id") {
    uri = `${env.NEXT_PUBLIC_DEPLOYED_URL}/api/worlds/${id.value}`;
    const metadata = await getWorldJson(id.value);
    return { metadata, uri };
  } else if (id.type === "uri") {
    return await fetchWorldMetadata(id.value);
  }

  return null;
});

import { cache } from "react";

import { env } from "@/src/env.mjs";

import { fetchWorldMetadata } from "./fetchWorldMetadata";

export const fetchPlayerCount = cache(async (uri: string, host?: string) => {
  try {
    let spaceHost = host;

    if (!spaceHost) {
      const metadata = await fetchWorldMetadata(uri);
      if (!metadata) throw new Error("Failed to read space metadata");

      spaceHost = metadata.info?.host || env.NEXT_PUBLIC_DEFAULT_HOST;
    }

    const http = spaceHost.startsWith("localhost") ? "http" : "https";

    const res = await fetch(`${http}://${spaceHost}/player-count/${uri}`, {
      cache: "no-store",
    });
    if (!res.ok) throw new Error("Failed to fetch player count");

    const playerCountText = await res.text();
    const playerCount = parseInt(playerCountText);
    if (isNaN(playerCount)) throw new Error("Failed to parse player count");

    return playerCount;
  } catch {
    return 0;
  }
});

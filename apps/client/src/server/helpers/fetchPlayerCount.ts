import { cache } from "react";

import { env } from "@/src/env.mjs";

import { fetchWorldMetadata } from "./fetchWorldMetadata";

export const fetchPlayerCount = cache(async (uri: string, host?: string) => {
  try {
    let worldHost = host;

    if (!worldHost) {
      const world = await fetchWorldMetadata(uri);
      if (!world) throw new Error("Failed to read world metadata");

      worldHost = world.metadata.info?.host || env.NEXT_PUBLIC_DEFAULT_HOST;
    }

    const http = worldHost.startsWith("localhost") ? "http" : "https";

    const res = await fetch(`${http}://${worldHost}/player-count/${uri}`, {
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

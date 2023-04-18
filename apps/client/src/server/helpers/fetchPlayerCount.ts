import { HostAPI } from "@unavi/protocol";
import { cache } from "react";

import { readSpaceMetadata } from "./readSpaceMetadata";

export const fetchPlayerCount = cache(async (uri: string, host?: string) => {
  try {
    let spaceHost = host;

    if (!spaceHost) {
      const metadata = await readSpaceMetadata(uri);
      if (!metadata) throw new Error("Failed to read space metadata");

      spaceHost = metadata.host;
    }

    const http = spaceHost.startsWith("localhost") ? "http" : "https";

    const res = await fetch(`${http}://${spaceHost}/${HostAPI.playerCount(uri)}`, {
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

import { HostAPI } from "@wired-labs/protocol";
import { cache } from "react";

import { readSpaceMetadata } from "./readSpaceMetadata";

export const fetchPlayerCount = cache(async (uri: string) => {
  try {
    const metadata = await readSpaceMetadata(uri);
    if (!metadata) throw new Error("Failed to read space metadata");

    const res = await fetch(`${metadata.host}/${HostAPI.space(uri).playerCount}`, {
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

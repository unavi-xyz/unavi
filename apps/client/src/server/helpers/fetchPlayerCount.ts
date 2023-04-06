import { HostAPI } from "@wired-labs/protocol";
import { cache } from "react";

import { processSpaceURI } from "./processSpaceURI";
import { readSpaceMetadata } from "./readSpaceMetadata";

export const fetchPlayerCount = cache(async (uri: string) => {
  try {
    const spaceUri = await processSpaceURI(uri);
    if (!spaceUri) throw new Error("Failed to process space URI");

    const metadata = await readSpaceMetadata(spaceUri);
    if (!metadata) throw new Error("Failed to read space metadata");

    const response = await fetch(`${metadata.host}/${HostAPI.space(spaceUri).playerCount}`, {
      cache: "no-store",
    });

    if (!response.ok) throw new Error("Failed to fetch player count");

    const playerCountText = await response.text();
    const playerCount = parseInt(playerCountText);
    if (isNaN(playerCount)) throw new Error("Failed to parse player count");

    return playerCount;
  } catch (e) {
    console.warn(e);
    return 0;
  }
});

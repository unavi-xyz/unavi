import { HostAPI } from "@wired-labs/protocol";
import { ATTRIBUTE_TYPES } from "contracts";
import { cache } from "react";

import { env } from "../../env.mjs";
import { fetchSpaceMetadata } from "./fetchSpaceMetadata";

const DEFAULT_HOST =
  process.env.NODE_ENV === "development"
    ? "http://localhost:4000"
    : `https://${env.NEXT_PUBLIC_DEFAULT_HOST}`;

export const fetchPlayerCount = cache(async (id: number) => {
  try {
    const metadata = await fetchSpaceMetadata(id);
    if (!metadata) throw new Error("Failed to fetch space metadata");

    const hostAttribute = metadata.attributes?.find(
      (attr) => attr.trait_type === ATTRIBUTE_TYPES.HOST
    );

    const spaceHost = hostAttribute?.value ? `https://${hostAttribute.value}` : DEFAULT_HOST;
    const host = spaceHost ?? DEFAULT_HOST;

    const response = await fetch(`${host}/${HostAPI.space(id).playerCount}`, {
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

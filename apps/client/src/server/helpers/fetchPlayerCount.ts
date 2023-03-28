import { HostAPI } from "@wired-labs/protocol";
import { cache } from "react";

import { env } from "../../env.mjs";

const HOST_URL =
  process.env.NODE_ENV === "development"
    ? "http://localhost:4000"
    : `https://${env.NEXT_PUBLIC_DEFAULT_HOST}`;

export const fetchPlayerCount = cache(async (id: number) => {
  try {
    const response = await fetch(`${HOST_URL}/${HostAPI.space(id).playerCount}`, {
      cache: "no-store",
    });
    const playerCountText = await response.text();
    const playerCount = parseInt(playerCountText);
    return isNaN(playerCount) ? 0 : playerCount;
  } catch {
    return 0;
  }
});

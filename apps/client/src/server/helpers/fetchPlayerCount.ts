import { env } from "../../env/server.mjs";

const HOST_URL =
  process.env.NODE_ENV === "development"
    ? "http://localhost:4000"
    : `https://${env.NEXT_PUBLIC_DEFAULT_HOST}`;

export async function fetchPlayerCount(id: number) {
  const response = await fetch(`${HOST_URL}/playercount/${id}`, { cache: "no-store" });
  const playerCountText = await response.text();
  return parseInt(playerCountText);
}

import useSWR from "swr";

import { PostPlayerCountArgs, PostPlayerCountResponse } from "./types";

export async function getPlayerCount(args: PostPlayerCountArgs) {
  const res = await fetch("/api/player-count", {
    body: JSON.stringify(args),
    cache: "no-store",
    headers: { "Content-Type": "application/json" },
    method: "POST",
  });
  if (!res.ok) throw new Error("Failed to fetch player count");

  const json = (await res.json()) as PostPlayerCountResponse;
  return json;
}

export function usePlayerCount(args: PostPlayerCountArgs) {
  const { data, error } = useSWR<PostPlayerCountResponse, Error>(
    JSON.stringify(args),
    () => getPlayerCount(args),
    {
      refreshInterval: 1000 * 60,
    }
  );

  return {
    isError: error,
    isLoading: !error && !data,
    playerCount: data?.count,
  };
}

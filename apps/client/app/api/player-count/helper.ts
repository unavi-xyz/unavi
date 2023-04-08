import useSWR from "swr";

import { PostPlayerCountArgs, PostPlayerCountResponse } from "./types";

export async function getPlayerCount(args: PostPlayerCountArgs) {
  const res = await fetch("/api/player-count", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    cache: "no-store",
    body: JSON.stringify(args),
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
    playerCount: data?.count,
    isLoading: !error && !data,
    isError: error,
  };
}

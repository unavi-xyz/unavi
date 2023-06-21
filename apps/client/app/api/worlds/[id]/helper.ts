import { GetResponse } from "./types";

export async function getWorld(id: string) {
  const response = await fetch(`/api/worlds/${id}`, { method: "GET" });
  const world = (await response.json()) as GetResponse;
  return world;
}

export function deleteWorld(id: string) {
  return fetch(`/api/worlds/${id}`, { method: "DELETE" });
}

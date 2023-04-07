import { GetSpaceResponse } from "./types";

export async function getSpace(id: string) {
  const response = await fetch(`/api/spaces/${id}`, { method: "GET" });
  const space = (await response.json()) as GetSpaceResponse;
  return space;
}

export function deleteSpace(id: string) {
  return fetch(`/api/spaces/${id}`, { method: "DELETE" });
}

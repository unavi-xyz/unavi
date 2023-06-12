import { GetSpaceResponse, PatchSpaceRequest } from "./types";

export async function getSpace(id: string) {
  const response = await fetch(`/api/spaces/${id}`, { method: "GET" });
  const space = (await response.json()) as GetSpaceResponse;
  return space;
}

export function updateSpace(id: string, data: PatchSpaceRequest) {
  return fetch(`/api/spaces/${id}`, {
    body: JSON.stringify(data),
    headers: { "Content-Type": "application/json" },
    method: "PATCH",
  });
}

export function deleteSpace(id: string) {
  return fetch(`/api/spaces/${id}`, { method: "DELETE" });
}

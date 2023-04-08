import { GetSpaceResponse, PatchSpaceRequest } from "./types";

export async function getSpace(id: string) {
  const response = await fetch(`/api/spaces/${id}`, { method: "GET" });
  const space = (await response.json()) as GetSpaceResponse;
  return space;
}

export function updateSpace(id: string, data: PatchSpaceRequest) {
  return fetch(`/api/spaces/${id}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export function deleteSpace(id: string) {
  return fetch(`/api/spaces/${id}`, { method: "DELETE" });
}

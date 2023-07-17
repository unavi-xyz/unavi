import { PatchBody } from "./types";

export function deleteWorld(id: string) {
  return fetch(`/api/worlds/${id}`, { method: "DELETE" });
}

export function updateWorld(id: string, body: PatchBody) {
  return fetch(`/api/worlds/${id}`, {
    body: JSON.stringify(body),
    method: "PATCH",
  });
}

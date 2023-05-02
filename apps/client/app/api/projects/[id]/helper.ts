import { PatchSchema } from "./types";

export function updateProject(id: string, data: PatchSchema) {
  return fetch(`/api/projects/${id}`, { body: JSON.stringify(data), method: "PATCH" });
}

export function deleteProject(id: string) {
  return fetch(`/api/projects/${id}`, { method: "DELETE" });
}

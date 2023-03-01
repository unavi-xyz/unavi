import { PatchSchema } from "./types";

export function updateProject(id: string, data: PatchSchema) {
  return fetch(`/api/projects/${id}`, { method: "PATCH", body: JSON.stringify(data) });
}

export function deleteProject(id: string) {
  return fetch(`/api/projects/${id}`, { method: "DELETE" });
}

import { PatchSchema } from "./types";

export function updateProject(id: string, data: PatchSchema) {
  return fetch(`/api/project/${id}`, { method: "PATCH", body: JSON.stringify(data) });
}

export function deleteProject(id: string) {
  return fetch(`/api/project/${id}`, { method: "DELETE" });
}

import { PublishProjectResponse } from "./types";

export async function publishProject(id: string) {
  const response = await fetch(`/api/projects/${id}/publication`, { method: "POST" });
  if (!response.ok) throw new Error(response.statusText);

  const json = (await response.json()) as PublishProjectResponse;
  return json;
}

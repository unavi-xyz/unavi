import { PublishProjectResponse } from "./types";

export async function publishProject(id: string) {
  const response = await fetch(`/api/project/${id}/publication`, { method: "POST" });
  const publication = (await response.json()) as PublishProjectResponse;
  return publication.id;
}

import { PostSchema, PublishProjectResponse } from "./types";

export async function publishProject(id: string, args: PostSchema = { optimize: true }) {
  const response = await fetch(`/api/projects/${id}/publication`, {
    method: "POST",
    body: JSON.stringify(args),
  });
  const json = (await response.json()) as PublishProjectResponse;
  return json;
}

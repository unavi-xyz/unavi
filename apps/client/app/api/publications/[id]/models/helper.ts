import { PostModelsResponse } from "./types";

export async function createPublishedModel(id: string) {
  const res = await fetch(`/api/publications/${id}/models`, { method: "POST" });
  if (!res.ok) throw new Error(res.statusText);

  const { modelId } = (await res.json()) as PostModelsResponse;
  return modelId;
}

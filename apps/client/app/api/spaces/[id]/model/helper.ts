import { PostSpaceModelResponse } from "./types";

export async function createSpaceModel(id: string) {
  const res = await fetch(`/api/spaces/${id}/model`, { method: "POST" });
  if (!res.ok) throw new Error(res.statusText);

  return (await res.json()) as PostSpaceModelResponse;
}

import { PostWorldModelResponse } from "./types";

export async function createWorldModel(id: string) {
  const res = await fetch(`/api/worlds/${id}/model`, { method: "POST" });
  if (!res.ok) throw new Error(res.statusText);

  return (await res.json()) as PostWorldModelResponse;
}

import { PostParams } from "./types";

export async function copyProjectToModel(spaceId: string, args: PostParams) {
  const response = await fetch(`/api/spaces/${spaceId}/model/copy-project`, {
    body: JSON.stringify(args),
    method: "POST",
  });

  if (!response.ok) throw new Error(response.statusText);
}

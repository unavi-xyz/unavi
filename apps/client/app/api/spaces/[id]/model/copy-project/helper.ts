import { PostParams } from "./types";

export async function copyProjectToModel(spaceId: string, args: PostParams) {
  const response = await fetch(`/api/spaces/${spaceId}/model/copy-project`, {
    method: "POST",
    body: JSON.stringify(args),
  });

  if (!response.ok) throw new Error(response.statusText);
}

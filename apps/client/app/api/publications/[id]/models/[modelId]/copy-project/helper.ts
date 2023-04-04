import { PostParams } from "./types";

export async function copyProjectToModel(publicationId: string, modelId: string, args: PostParams) {
  const response = await fetch(
    `/api/publications/${publicationId}/models/${modelId}/copy-project`,
    {
      method: "POST",
      body: JSON.stringify(args),
    }
  );

  if (!response.ok) throw new Error(response.statusText);
}

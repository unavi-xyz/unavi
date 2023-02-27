import { GetModelUploadResponse } from "./types";

export async function getProjectModelUpload(id: string) {
  const response = await fetch(`/api/project/${id}/model/upload`, { method: "GET" });
  const { url } = (await response.json()) as GetModelUploadResponse;
  return url;
}

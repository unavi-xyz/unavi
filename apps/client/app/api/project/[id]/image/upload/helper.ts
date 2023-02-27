import { GetImageUploadResponse } from "./types";

export async function getProjectImageUpload(id: string) {
  const response = await fetch(`/api/project/${id}/image/upload`, { method: "GET" });
  const { url } = (await response.json()) as GetImageUploadResponse;
  return url;
}

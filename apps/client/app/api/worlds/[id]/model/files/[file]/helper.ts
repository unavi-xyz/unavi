import { WorldModelFile } from "./files";
import { GetFileDownloadResponse, GetFileUploadResponse } from "./types";

export async function getWorldModelFileDownload(
  id: string,
  file: WorldModelFile
) {
  const response = await fetch(`/api/worlds/${id}/model/files/${file}`, {
    method: "GET",
  });
  const { url } = (await response.json()) as GetFileDownloadResponse;
  return url;
}

export async function getWorldModelFileUpload(
  id: string,
  file: WorldModelFile
) {
  const response = await fetch(`/api/worlds/${id}/model/files/${file}`, {
    method: "PUT",
  });
  const { url } = (await response.json()) as GetFileUploadResponse;
  return url;
}

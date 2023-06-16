import { SpaceModelFile } from "./files";
import { GetFileDownloadResponse, GetFileUploadResponse } from "./types";

export async function getSpaceModelFileDownload(
  id: string,
  file: SpaceModelFile
) {
  const response = await fetch(`/api/spaces/${id}/model/files/${file}`, {
    method: "GET",
  });
  const { url } = (await response.json()) as GetFileDownloadResponse;
  return url;
}

export async function getSpaceModelFileUpload(
  id: string,
  file: SpaceModelFile
) {
  const response = await fetch(`/api/spaces/${id}/model/files/${file}`, {
    method: "PUT",
  });
  const { url } = (await response.json()) as GetFileUploadResponse;
  return url;
}

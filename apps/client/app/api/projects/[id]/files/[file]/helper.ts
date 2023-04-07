import { ProjectFile } from "./files";
import { GetFileDownloadResponse, GetFileUploadResponse } from "./types";

export async function getProjectFileDownload(id: string, file: ProjectFile) {
  const response = await fetch(`/api/projects/${id}/files/${file}`, { method: "GET" });
  const { url } = (await response.json()) as GetFileDownloadResponse;
  return url;
}

export async function getProjectFileUpload(id: string, file: ProjectFile) {
  const response = await fetch(`/api/projects/${id}/files/${file}`, { method: "PUT" });
  const { url } = (await response.json()) as GetFileUploadResponse;
  return url;
}

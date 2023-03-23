import { ProjectFile } from "../../files";
import { GetFileDownloadResponse, GetFileUploadResponse } from "./types";

export async function getProjectFileDownload(id: string, file: ProjectFile) {
  const response = await fetch(`/api/projects/${id}/${file}`, { method: "GET" });
  const { url } = (await response.json()) as GetFileDownloadResponse;
  return url;
}

export async function getProjectFileUpload(id: string, file: ProjectFile) {
  const response = await fetch(`/api/projects/${id}/${file}/upload`, { method: "PUT" });
  const { url } = (await response.json()) as GetFileUploadResponse;
  return url;
}

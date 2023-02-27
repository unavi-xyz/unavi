import { ProjectFile } from "../../files";
import { GetFileDownloadResponse } from "./types";

export async function getProjectFileDownload(id: string, file: ProjectFile) {
  const response = await fetch(`/api/project/${id}/${file}`, { method: "GET" });
  const { url } = (await response.json()) as GetFileDownloadResponse;
  return url;
}

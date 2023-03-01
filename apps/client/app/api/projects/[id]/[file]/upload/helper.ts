import { ProjectFile } from "../../../files";
import { GetFileUploadResponse } from "./types";

export async function getProjectFileUpload(id: string, file: ProjectFile) {
  const response = await fetch(`/api/projects/${id}/${file}/upload`, { method: "GET" });
  const { url } = (await response.json()) as GetFileUploadResponse;
  return url;
}

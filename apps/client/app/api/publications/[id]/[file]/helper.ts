import { PublicationFile } from "../../files";
import { GetFileDownloadResponse } from "./types";

export async function getPublicationFileDownload(id: string, file: PublicationFile) {
  const response = await fetch(`/api/publications/${id}/${file}`, { method: "GET" });
  const { url } = (await response.json()) as GetFileDownloadResponse;
  return url;
}

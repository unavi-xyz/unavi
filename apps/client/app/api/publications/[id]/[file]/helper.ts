import { PublicationFile } from "../../files";
import { GetFileUploadResponse } from "./types";

export async function getPublicationFileUpload(id: string, file: PublicationFile) {
  const response = await fetch(`/api/publications/${id}/${file}/upload`, { method: "PUT" });
  const { url } = (await response.json()) as GetFileUploadResponse;
  return url;
}

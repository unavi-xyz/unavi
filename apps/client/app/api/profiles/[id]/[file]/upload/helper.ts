import { ProfileFile } from "../../../files";
import { GetFileUploadResponse } from "./types";

export async function getProfileFileUpload(id: number, file: ProfileFile) {
  const response = await fetch(`/api/profiles/${id}/${file}/upload`, { method: "GET" });
  const { url } = (await response.json()) as GetFileUploadResponse;
  return url;
}

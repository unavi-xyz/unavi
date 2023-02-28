import { ProfileFile } from "../../files";
import { GetFileDownloadResponse } from "./types";

export async function getProfileFileDownload(id: number, file: ProfileFile) {
  const response = await fetch(`/api/profile/${id}/${file}`, { method: "GET" });
  const { url } = (await response.json()) as GetFileDownloadResponse;
  return url;
}

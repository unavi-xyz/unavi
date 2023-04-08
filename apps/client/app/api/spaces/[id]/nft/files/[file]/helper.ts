import { SpaceNFTFile } from "./files";
import { GetFileDownloadResponse, GetFileUploadResponse } from "./types";

export async function getSpaceNFTFileDownload(id: string, file: SpaceNFTFile) {
  const response = await fetch(`/api/spaces/${id}/nft/files/${file}`, { method: "GET" });
  const { url } = (await response.json()) as GetFileDownloadResponse;
  return url;
}

export async function getSpaceNFTFileUpload(id: string, file: SpaceNFTFile) {
  const response = await fetch(`/api/spaces/${id}/nft/files/${file}`, { method: "PUT" });
  const { url } = (await response.json()) as GetFileUploadResponse;
  return url;
}

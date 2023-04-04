import { PublishedModelFile } from "../../files";
import { GetFileUploadResponse } from "./types";

export async function getPublishedModelFileUpload(
  publicationId: string,
  modelId: string,
  file: PublishedModelFile
) {
  const response = await fetch(
    `/api/publications/${publicationId}/models/${modelId}/files/${file}`,
    {
      method: "PUT",
    }
  );
  const { url } = (await response.json()) as GetFileUploadResponse;
  return url;
}

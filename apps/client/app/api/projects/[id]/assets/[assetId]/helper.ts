import { PutAssetResponse } from "./types";

export async function getProjectAssetUpload(id: string, assetId: string) {
  const response = await fetch(`/api/projects/${id}/assets/${assetId}`, { method: "PUT" });
  return (await response.json()) as PutAssetResponse;
}

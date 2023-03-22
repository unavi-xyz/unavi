import { PostAssetsResponse } from "./types";

export async function getNewProjectAssetUpload(id: string) {
  const response = await fetch(`/api/projects/${id}/assets`, { method: "POST" });
  const { url, assetId } = (await response.json()) as PostAssetsResponse;
  return { url, assetId };
}

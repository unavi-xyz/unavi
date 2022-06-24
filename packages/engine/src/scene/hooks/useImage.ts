import { useAsset } from "./useAsset";

export function useImage(id: string | undefined) {
  return useAsset<"image">(id)?.data;
}

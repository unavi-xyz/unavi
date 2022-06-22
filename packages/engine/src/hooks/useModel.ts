import { useAsset } from "./useAsset";

export function useModel(id: string | undefined) {
  return useAsset<"model">(id)?.data;
}

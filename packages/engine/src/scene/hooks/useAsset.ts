import { useContext } from "react";

import { AssetContext } from "../components/scene/AssetProvider";
import { Asset, AssetType } from "../types";

export function useAsset<T extends AssetType = AssetType>(id: string | undefined) {
  const { assets } = useContext(AssetContext);
  if (!id) return;
  const asset = assets[id];
  if (!asset) return;
  return asset as Asset<T>;
}

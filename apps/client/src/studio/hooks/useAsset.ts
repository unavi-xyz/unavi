import { useEffect, useState } from "react";

import { Asset, AssetType } from "@wired-xr/engine";

import { useStudioStore } from "../store";

export function useAsset<T extends AssetType = AssetType>(
  assetId: string | undefined
) {
  const [asset, setAsset] = useState<Asset>();
  const assets = useStudioStore((state) => state.scene.assets);

  useEffect(() => {
    if (!assetId) {
      setAsset(undefined);
      return;
    }

    const newAsset = assets[assetId];
    setAsset(newAsset);
  }, [assetId, assets]);

  if (!asset) return;

  return asset as Asset<T>;
}

export function useMaterialAsset(assetId: string | undefined) {
  const asset = useAsset<"material">(assetId);
  return asset;
}

export function useImageAsset(assetId: string | undefined) {
  const asset = useAsset<"image">(assetId);
  return asset;
}

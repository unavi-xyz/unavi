import { env } from "../../env/client.mjs";

export function pathAsset(assetId: string) {
  return `assets/${assetId}` as const;
}

type Path = ReturnType<typeof pathAsset>;

export function cdnURL(path: Path) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/${path}`;
}

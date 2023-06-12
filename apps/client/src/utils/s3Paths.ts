import { env } from "../env.mjs";

export class S3Path {
  static spaceNFT = (nftId: string) =>
    ({
      directory: `nfts/${nftId}`,
      metadata: `nfts/${nftId}/metadata.json`,
    } as const);

  static profile = (userId: string) => {
    return {
      background: (fileId: string) => `profiles/${userId}/background/${fileId}` as const,
      image: (fileId: string) => `profiles/${userId}/image/${fileId}` as const,
    } as const;
  };

  static project = (projectId: string) =>
    ({
      asset: (assetId: string) => `projects/${projectId}/assets/${assetId}` as const,
      assets: `projects/${projectId}/assets`,
      directory: `projects/${projectId}`,
      image: `projects/${projectId}/image`,
      model: `projects/${projectId}/model.glb`,
    } as const);

  static spaceModel = (modelId: string) =>
    ({
      asset: (assetId: string) => `spaces/${modelId}/assets/${assetId}` as const,
      directory: `spaces/${modelId}`,
      image: `spaces/${modelId}/image`,
      metadata: `spaces/${modelId}/metadata.json`,
      model: `spaces/${modelId}/model.glb`,
    } as const);

  static temp(fileId: string) {
    return `temp/${fileId}` as const;
  }
}

type PublicPath =
  | ReturnType<typeof S3Path.spaceNFT>[keyof ReturnType<typeof S3Path.spaceNFT>]
  | ReturnType<typeof S3Path.spaceModel>["metadata" | "model" | "image"]
  | ReturnType<ReturnType<typeof S3Path.spaceModel>["asset"]>
  | ReturnType<ReturnType<typeof S3Path.profile>["background"]>
  | ReturnType<ReturnType<typeof S3Path.profile>["image"]>
  | ReturnType<typeof S3Path.temp>;

export function cdnURL(path: PublicPath) {
  // Use http on localhost
  const http =
    env.NEXT_PUBLIC_CDN_ENDPOINT?.startsWith("localhost") ||
    env.NEXT_PUBLIC_CDN_ENDPOINT?.startsWith("127.0.0.1")
      ? "http"
      : "https";

  return `${http}://${env.NEXT_PUBLIC_CDN_ENDPOINT}/${path}`;
}

import { env } from "../env.mjs";
import { toHex } from "./toHex";

export class S3Path {
  static spaceNFT = (nftId: string) =>
    ({
      directory: `nfts/${nftId}`,
      metadata: `nfts/${nftId}/metadata.json`,
    } as const);

  static profile = (profileId: number) => {
    const hexId = toHex(profileId);
    return {
      cover: `profiles/${hexId}/cover.jpg`,
      image: `profiles/${hexId}/image.jpg`,
      metadata: `profiles/${hexId}/metadata.json`,
    } as const;
  };

  static project = (projectId: string) =>
    ({
      asset: (assetId: string) => `projects/${projectId}/assets/${assetId}` as const,
      assets: `projects/${projectId}/assets`,
      directory: `projects/${projectId}`,
      image: `projects/${projectId}/image.jpg`,
      model: `projects/${projectId}/model.glb`,
    } as const);

  static spaceModel = (modelId: string) =>
    ({
      directory: `spaces/${modelId}`,
      metadata: `spaces/${modelId}/metadata.json`,
      model: `spaces/${modelId}/model.glb`,
      image: `spaces/${modelId}/image.jpg`,
      asset: (assetId: string) => `spaces/${modelId}/assets/${assetId}` as const,
    } as const);

  static temp(fileId: string) {
    return `temp/${fileId}` as const;
  }
}

type PublicPath =
  | ReturnType<typeof S3Path.spaceNFT>[keyof ReturnType<typeof S3Path.spaceNFT>]
  | ReturnType<typeof S3Path.spaceModel>["model" | "image"]
  | ReturnType<ReturnType<typeof S3Path.spaceModel>["asset"]>
  | ReturnType<typeof S3Path.profile>[keyof ReturnType<typeof S3Path.profile>]
  | ReturnType<typeof S3Path.temp>;

export function cdnURL(path: PublicPath) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/${path}`;
}

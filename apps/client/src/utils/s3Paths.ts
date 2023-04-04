import { env } from "../env.mjs";
import { toHex } from "./toHex";

export class S3Path {
  static publication = (publicationId: string) =>
    ({
      metadata: `publications/${publicationId}/metadata.json`,
      model: (modelId: string) =>
        ({
          directory: `publications/${publicationId}/models/${modelId}`,
          image: `publications/${publicationId}/models/${modelId}/image.jpg`,
          model: `publications/${publicationId}/models/${modelId}/model.glb`,
          asset: (assetId: string) =>
            `publications/${publicationId}/models/${modelId}/assets/${assetId}` as const,
        } as const),
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
      image: `projects/${projectId}/image.jpg`,
      model: `projects/${projectId}/model.glb`,
    } as const);

  static temp(fileId: string) {
    return `temp/${fileId}` as const;
  }
}

type PublicPath =
  | ReturnType<typeof S3Path.publication>["metadata"]
  | ReturnType<ReturnType<typeof S3Path.publication>["model"]>["image" | "model"]
  | ReturnType<ReturnType<ReturnType<typeof S3Path.publication>["model"]>["asset"]>
  | ReturnType<typeof S3Path.profile>[keyof ReturnType<typeof S3Path.profile>]
  | ReturnType<typeof S3Path.temp>;

export function cdnURL(path: PublicPath) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/${path}`;
}

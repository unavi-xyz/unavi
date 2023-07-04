import { env } from "../env.mjs";

export class S3Path {
  static profile = (userKey: string) => {
    return {
      background: (fileKey: string) =>
        `profiles/${userKey}/background/${fileKey}` as const,
      image: (fileKey: string) =>
        `profiles/${userKey}/image/${fileKey}` as const,
    } as const;
  };

  static project = (projectKey: string) =>
    ({
      asset: (assetKey: string) =>
        `projects/${projectKey}/assets/${assetKey}` as const,
      assets: `projects/${projectKey}/assets`,
      directory: `projects/${projectKey}`,
      image: `projects/${projectKey}/image`,
      model: `projects/${projectKey}/model.glb`,
    } as const);

  static worldModel = (modelKey: string) =>
    ({
      asset: (assetKey: string) =>
        `worlds/${modelKey}/assets/${assetKey}` as const,
      directory: `worlds/${modelKey}`,
      image: `worlds/${modelKey}/image`,
      model: `worlds/${modelKey}/model.glb`,
    } as const);

  static temp(fileKey: string) {
    return `temp/${fileKey}` as const;
  }
}

type PublicPath =
  | ReturnType<typeof S3Path.worldModel>["model" | "image"]
  | ReturnType<ReturnType<typeof S3Path.worldModel>["asset"]>
  | ReturnType<ReturnType<typeof S3Path.profile>["background"]>
  | ReturnType<ReturnType<typeof S3Path.profile>["image"]>
  | ReturnType<typeof S3Path.temp>;

const http =
  env.NEXT_PUBLIC_CDN_ENDPOINT?.startsWith("localhost") ||
  env.NEXT_PUBLIC_CDN_ENDPOINT?.startsWith("127.0.0.1")
    ? "http"
    : "https";

export function cdnURL(path: PublicPath) {
  return `${http}://${env.NEXT_PUBLIC_CDN_ENDPOINT}/${path}`;
}

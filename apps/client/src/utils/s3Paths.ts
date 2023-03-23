import { env } from "../env/client.mjs";
import { toHex } from "./toHex";

export class S3Path {
  static asset(assetId: string) {
    return `assets/${assetId}` as const;
  }

  static publication = (publicationId: string) =>
    ({
      image: `publications/${publicationId}/image.jpg`,
      metadata: `publications/${publicationId}/metadata.json`,
      model: `publications/${publicationId}/model.glb`,
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
      image: `projects/${projectId}/image.jpg`,
      model: `projects/${projectId}/model.glb`,
    } as const);
}

type PublicPath =
  | ReturnType<typeof S3Path.asset>
  | ReturnType<typeof S3Path.publication>[keyof ReturnType<typeof S3Path.publication>]
  | ReturnType<typeof S3Path.profile>[keyof ReturnType<typeof S3Path.profile>];

export function cdnURL(path: PublicPath) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/${path}`;
}

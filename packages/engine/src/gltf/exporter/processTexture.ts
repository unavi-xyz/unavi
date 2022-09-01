import { Texture } from "three";

import { GLTF, Texture as TextureDef } from "../schemaTypes";

export function processTexture(
  map: Texture,
  json: GLTF,
  pending: Promise<any>[],
  processSampler: (map: Texture) => number,
  processImage: (
    image: any,
    format: number,
    flipY: boolean,
    mimeType?: string
  ) => Promise<number>
): number {
  let mimeType: string | undefined = map.userData.mimeType;
  if (mimeType === "image/webp") mimeType = "image/png";

  const sampler = processSampler(map);
  const textureDef: TextureDef = { sampler };

  const promise = new Promise<void>((resolve) => {
    // Wait for all pending promises to finish
    // We need an up to date byteLength for creating the image buffer view
    // This is weird and hacky, but it works
    Promise.all(pending).then(() => {
      const sourcePromise = processImage(
        map.image,
        map.format,
        map.flipY,
        mimeType
      );

      sourcePromise.then((source) => {
        textureDef.source = source;
        resolve();
      });
    });
  });

  pending.push(promise);

  if (!json.textures) json.textures = [];
  const index = json.textures.push(textureDef) - 1;
  return index;
}

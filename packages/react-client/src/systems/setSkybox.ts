import { Asset } from "lattice-engine/core";
import { Image, Scene } from "lattice-engine/scene";
import { Entity, Mut, Query } from "thyseus";

import { config } from "../config";

export function setSkybox(
  images: Query<[Entity, Mut<Asset>, Mut<Image>]>,
  scenes: Query<Scene>
) {
  for (const scene of scenes) {
    for (const [entity, asset, image] of images) {
      if (scene.skyboxId !== entity.id) continue;
      if (asset.uri === config.skyboxUri) continue;

      asset.uri = config.skyboxUri;
      image.flipY = true;

      const fileExt = asset.uri.split(".").pop();

      switch (fileExt) {
        case "jpg":
        case "jpeg": {
          asset.mimeType = "image/jpeg";
          break;
        }

        default: {
          asset.mimeType = "";
          break;
        }
      }
    }
  }
}

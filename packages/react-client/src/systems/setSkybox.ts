import { Asset } from "lattice-engine/core";
import { Image, Scene } from "lattice-engine/scene";
import { Entity, Mut, Query } from "thyseus";

import { useClientStore } from "../store";

export function setSkybox(
  images: Query<[Entity, Mut<Asset>, Mut<Image>]>,
  scenes: Query<Scene>
) {
  for (const scene of scenes) {
    for (const [entity, asset, image] of images) {
      if (scene.skyboxId !== entity.id) continue;

      const skybox = useClientStore.getState().skybox;
      if (asset.uri === skybox) continue;

      asset.uri = skybox;
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

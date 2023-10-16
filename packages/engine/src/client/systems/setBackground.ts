import { Asset } from "houseki/core";
import { Background, Image } from "houseki/scene";
import { Entity, Mut, Query } from "thyseus";

import { useClientStore } from "../clientStore";

export function setBackground(
  backgrounds: Query<Background>,
  images: Query<[Entity, Mut<Asset>, Mut<Image>]>
) {
  for (const background of backgrounds) {
    for (const [entity, asset, image] of images) {
      if (background.imageId !== entity.id) continue;

      const skyboxUri = useClientStore.getState().skybox;
      if (asset.uri === skyboxUri) continue;

      asset.uri = skyboxUri;
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

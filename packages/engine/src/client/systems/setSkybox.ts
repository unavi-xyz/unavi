import { atom, getDefaultStore } from "jotai";
import { Asset } from "lattice-engine/core";
import { Image, Skybox } from "lattice-engine/scene";
import { Entity, Mut, Query } from "thyseus";

export const skyboxAtom = atom("");

export function setSkybox(
  skyboxes: Query<Skybox>,
  images: Query<[Entity, Mut<Asset>, Mut<Image>]>
) {
  for (const skybox of skyboxes) {
    for (const [entity, asset, image] of images) {
      if (skybox.imageId !== entity.id) continue;

      const skyboxUri = getDefaultStore().get(skyboxAtom);

      const uri = asset.uri;
      if (uri === skyboxUri) continue;

      asset.uri = skyboxUri;
      image.flipY = true;

      const fileExt = uri.split(".").pop();

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

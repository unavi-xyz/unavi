import { atom, getDefaultStore } from "jotai";
import { Asset, Warehouse } from "lattice-engine/core";
import { Image, Skybox } from "lattice-engine/scene";
import { Entity, Mut, Query, Res } from "thyseus";

export const skyboxAtom = atom("");

export function setSkybox(
  warehouse: Res<Mut<Warehouse>>,
  skyboxes: Query<Skybox>,
  images: Query<[Entity, Mut<Asset>, Mut<Image>]>
) {
  for (const skybox of skyboxes) {
    for (const [entity, asset, image] of images) {
      if (skybox.imageId !== entity.id) continue;

      const skyboxUri = getDefaultStore().get(skyboxAtom);

      const uri = asset.uri.read(warehouse) ?? "";
      if (uri === skyboxUri) continue;

      asset.uri.write(skyboxUri, warehouse);
      image.flipY = true;

      const fileExt = uri.split(".").pop();

      switch (fileExt) {
        case "jpg":
        case "jpeg": {
          asset.mimeType.write("image/jpeg", warehouse);
          break;
        }

        default: {
          asset.mimeType.write("", warehouse);
          break;
        }
      }
    }
  }
}

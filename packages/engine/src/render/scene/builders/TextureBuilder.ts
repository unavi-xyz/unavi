import { Texture } from "@gltf-transform/core";
import { Texture as ThreeTexture } from "three";

import { TextureJSON } from "../../../scene";
import { subscribe } from "../../../utils/subscribe";
import { Builder } from "./Builder";

/**
 * @internal
 * Handles the conversion of textures to Three.js objects.
 */
export class TextureBuilder extends Builder<Texture, TextureJSON, ThreeTexture> {
  add(json: Partial<TextureJSON>, id: string) {
    const previousObject = this.getObject(id);
    if (previousObject) throw new Error(`Texture with id ${id} already exists.`);

    const { object: texture } = this.scene.texture.create(json, id);

    const object = new ThreeTexture();

    this.setObject(id, object);

    subscribe(texture, "Name", (value) => {
      object.name = value;
    });

    subscribe(texture, "Image", (array) => {
      if (!array) {
        object.image = null;
        object.needsUpdate = true;
        return;
      }

      return subscribe(texture, "MimeType", async (type) => {
        const blob = new Blob([array], { type });
        const bitmap = await createImageBitmap(blob);

        object.image = bitmap;
        object.needsUpdate = true;
      });
    });

    texture.addEventListener("dispose", () => {
      object.dispose();
      this.setObject(id, null);
    });

    return texture;
  }

  remove(id: string) {
    this.scene.texture.store.get(id)?.dispose();
  }

  update(json: Partial<TextureJSON>, id: string) {
    const texture = this.scene.texture.store.get(id);
    if (!texture) throw new Error(`Texture with id ${id} does not exist.`);

    this.scene.texture.applyJSON(texture, json);
  }
}

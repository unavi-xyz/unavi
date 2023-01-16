import { Document, Texture } from "@gltf-transform/core";
import { nanoid } from "nanoid";

import { Utils } from "./Utils";

export interface TextureJSON {
  image: Uint8Array | null;
  mimeType: string;
  uri: string;
}

export class TextureUtils extends Utils<Texture, TextureJSON> {
  #doc: Document;

  store = new Map<string, Texture>();

  constructor(doc: Document) {
    super();

    this.#doc = doc;
  }

  getId(texture: Texture) {
    for (const [id, m] of this.store) {
      if (m === texture) return id;
    }
  }

  create(json: Partial<TextureJSON> = {}, id?: string) {
    const texture = this.#doc.createTexture();
    this.applyJSON(texture, json);

    const { id: textureId } = this.process(texture, id);

    this.emitCreate(textureId);

    return { id: textureId, object: texture };
  }

  process(texture: Texture, id?: string) {
    const textureId = id ?? nanoid();
    this.store.set(textureId, texture);

    texture.addEventListener("dispose", () => {
      this.store.delete(textureId);
    });

    return { id: textureId };
  }

  processChanges() {
    const changed: Texture[] = [];

    // Add new textures
    this.#doc
      .getRoot()
      .listTextures()
      .forEach((texture) => {
        const textureId = this.getId(texture);
        if (textureId) return;

        this.process(texture);
        changed.push(texture);
      });

    return changed;
  }

  applyJSON(texture: Texture, json: Partial<TextureJSON>) {
    if (json.image) texture.setImage(json.image);
    if (json.mimeType) texture.setMimeType(json.mimeType);
    if (json.uri) texture.setURI(json.uri);
  }

  toJSON(texture: Texture): TextureJSON {
    return {
      image: texture.getImage(),
      mimeType: texture.getMimeType(),
      uri: texture.getURI(),
    };
  }
}

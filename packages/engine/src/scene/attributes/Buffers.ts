import { Buffer, Document } from "@gltf-transform/core";
import { nanoid } from "nanoid";

import { Attribute } from "./Attribute";

export interface BufferJSON {
  uri: string;
}

/**
 * Stores and manages buffers within a {@link Scene}.
 *
 * @group Scene
 */
export class Buffers extends Attribute<Buffer, BufferJSON> {
  #doc: Document;

  store = new Map<string, Buffer>();

  constructor(doc: Document) {
    super();

    this.#doc = doc;
  }

  getId(buffer: Buffer) {
    for (const [id, m] of this.store) {
      if (m === buffer) return id;
    }
  }

  create(json: Partial<BufferJSON> = {}, id?: string) {
    const buffer = this.#doc.createBuffer();
    this.applyJSON(buffer, json);

    const { id: bufferId } = this.process(buffer, id);

    this.emitCreate(bufferId);

    return { id: bufferId, object: buffer };
  }

  process(buffer: Buffer, id?: string) {
    const bufferId = id ?? nanoid();
    this.store.set(bufferId, buffer);

    buffer.addEventListener("dispose", () => {
      this.store.delete(bufferId);
    });

    return { id: bufferId };
  }

  processChanges() {
    const changed: Buffer[] = [];

    // Add new buffers
    this.#doc
      .getRoot()
      .listBuffers()
      .forEach((buffer) => {
        const bufferId = this.getId(buffer);
        if (bufferId) return;

        this.process(buffer);
        changed.push(buffer);
      });

    return changed;
  }

  applyJSON(buffer: Buffer, json: Partial<BufferJSON>) {
    if (json.uri) buffer.setURI(json.uri);
  }

  toJSON(buffer: Buffer): BufferJSON {
    return {
      uri: buffer.getURI(),
    };
  }
}

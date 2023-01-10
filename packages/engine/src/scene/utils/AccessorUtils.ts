import { Accessor, Document, GLTF, TypedArray } from "@gltf-transform/core";
import { nanoid } from "nanoid";

import { BufferUtils } from "./BufferUtils";
import { Utils } from "./types";

type BufferId = string;

export interface AccessorJSON {
  array: TypedArray | null;
  type: GLTF.AccessorType;
  componentType: GLTF.AccessorComponentType;
  normalized: boolean;
  buffer: BufferId | null;
}

export class AccessorUtils implements Utils<Accessor, AccessorJSON> {
  #doc: Document;
  #buffer: BufferUtils;

  store = new Map<string, Accessor>();

  constructor(doc: Document, buffer: BufferUtils) {
    this.#doc = doc;
    this.#buffer = buffer;
  }

  getId(accessor: Accessor) {
    for (const [id, m] of this.store) {
      if (m === accessor) return id;
    }
  }

  create(json: Partial<AccessorJSON> = {}, id?: string) {
    const accessor = this.#doc.createAccessor();
    this.applyJSON(accessor, json);

    const { id: accessorId } = this.process(accessor, id);

    return { id: accessorId, object: accessor };
  }

  process(accessor: Accessor, id?: string) {
    const accessorId = id ?? nanoid();
    this.store.set(accessorId, accessor);

    accessor.addEventListener("dispose", () => {
      this.store.delete(accessorId);
    });

    return { id: accessorId };
  }

  processChanges() {
    const changed: Accessor[] = [];

    // Add new accessors
    this.#doc
      .getRoot()
      .listAccessors()
      .forEach((accessor) => {
        const accessorId = this.getId(accessor);
        if (accessorId) return;

        this.process(accessor);
        changed.push(accessor);
      });

    return changed;
  }

  applyJSON(accessor: Accessor, json: Partial<AccessorJSON>) {
    if (json.array) accessor.setArray(json.array);
    if (json.type) accessor.setType(json.type);
    if (json.normalized) accessor.setNormalized(json.normalized);

    if (json.buffer) {
      const buffer = json.buffer ? this.#buffer.store.get(json.buffer) : null;
      if (buffer === undefined) throw new Error("Buffer not found");
      accessor.setBuffer(buffer);
    }
  }

  toJSON(accessor: Accessor): AccessorJSON {
    const buffer = accessor.getBuffer();
    const bufferId = buffer ? this.#buffer.getId(buffer) : null;
    if (bufferId === undefined) throw new Error("Buffer not found");

    return {
      array: accessor.getArray(),
      type: accessor.getType(),
      componentType: accessor.getComponentType(),
      normalized: accessor.getNormalized(),
      buffer: bufferId,
    };
  }
}

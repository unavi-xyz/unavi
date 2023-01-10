import { Document, Mesh } from "@gltf-transform/core";
import { nanoid } from "nanoid";

import { PrimitiveUtils } from "./PrimitiveUtils";
import { Utils } from "./types";

type PrimitiveId = string;

export interface MeshJSON {
  primitives: PrimitiveId[];
  weights: number[];
}

export class MeshUtils implements Utils<Mesh, MeshJSON> {
  #doc: Document;
  #primitive: PrimitiveUtils;

  store = new Map<string, Mesh>();

  constructor(doc: Document, primitive: PrimitiveUtils) {
    this.#doc = doc;
    this.#primitive = primitive;
  }

  getId(mesh: Mesh) {
    for (const [id, m] of this.store) {
      if (m === mesh) return id;
    }
  }

  create(json: Partial<MeshJSON> = {}, id?: string) {
    const mesh = this.#doc.createMesh();
    this.applyJSON(mesh, json);

    const { id: meshId } = this.process(mesh, id);

    return { id: meshId, object: mesh };
  }

  process(mesh: Mesh, id?: string) {
    const meshId = id ?? nanoid();
    this.store.set(meshId, mesh);

    mesh.addEventListener("dispose", () => {
      this.store.delete(meshId);
    });

    return { id: meshId };
  }

  processChanges() {
    const changed: Mesh[] = [];

    // Add new meshes
    this.#doc
      .getRoot()
      .listMeshes()
      .forEach((mesh) => {
        const meshId = this.getId(mesh);
        if (meshId) return;

        this.process(mesh);
        changed.push(mesh);
      });

    return changed;
  }

  applyJSON(mesh: Mesh, json: Partial<MeshJSON>) {
    if (json.primitives) {
      for (const primitiveId of json.primitives) {
        const primitive = this.#primitive.store.get(primitiveId);
        if (!primitive) throw new Error("Primitive not found");

        mesh.addPrimitive(primitive);
      }
    }

    if (json.weights) {
      mesh.setWeights(json.weights);
    }
  }

  toJSON(mesh: Mesh): MeshJSON {
    const primitiveIds = mesh.listPrimitives().map((primitive) => {
      const id = this.#primitive.getId(primitive);
      if (!id) throw new Error("Primitive not found");
      return id;
    });

    return {
      primitives: primitiveIds,
      weights: mesh.getWeights(),
    };
  }
}

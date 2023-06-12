import { Document, Mesh } from "@gltf-transform/core";
import { nanoid } from "nanoid";

import { Scene } from "../Scene";
import { Attribute } from "./Attribute";
import { Primitives } from "./Primitives";

type PrimitiveId = string;

type BoxMesh = {
  type: "Box";
  width: number;
  height: number;
  depth: number;
};

type SphereMesh = {
  type: "Sphere";
  radius: number;
  widthSegments: number;
  heightSegments: number;
};

type CylinderMesh = {
  type: "Cylinder";
  radiusTop: number;
  radiusBottom: number;
  height: number;
  radialSegments: number;
};

export type CustomMesh = BoxMesh | SphereMesh | CylinderMesh;

export type MeshExtras = {
  customMesh?: CustomMesh;
};

export interface MeshJSON {
  primitives: PrimitiveId[];
  weights: number[];
  extras?: MeshExtras;
}

/**
 * Stores and manages meshes within a {@link Scene}.
 *
 * @group Scene
 */
export class Meshes extends Attribute<Mesh, MeshJSON> {
  #doc: Document;
  #primitive: Primitives;

  store = new Map<string, Mesh>();

  constructor(scene: Scene) {
    super();

    this.#doc = scene.doc;
    this.#primitive = scene.primitive;
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

    this.emitCreate(meshId);

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
    if (json.primitives !== undefined) {
      // Remove old primitives
      mesh.listPrimitives().forEach((primitive) => {
        const primitiveId = this.#primitive.getId(primitive);
        if (!primitiveId) throw new Error("Primitive not found");

        if (!json.primitives?.includes(primitiveId)) {
          mesh.removePrimitive(primitive);
        }
      });

      // Add new primitives
      json.primitives.forEach((primitiveId) => {
        const primitive = this.#primitive.store.get(primitiveId);
        if (!primitive) throw new Error("Primitive not found");

        mesh.addPrimitive(primitive);
      });
    }

    if (json.weights !== undefined) mesh.setWeights(json.weights);
    if (json.extras !== undefined) mesh.setExtras(json.extras);
  }

  toJSON(mesh: Mesh): MeshJSON {
    const primitiveIds = mesh.listPrimitives().map((primitive) => {
      const id = this.#primitive.getId(primitive);
      if (!id) throw new Error("Primitive not found");
      return id;
    });

    return {
      extras: mesh.getExtras() as MeshExtras,
      primitives: primitiveIds,
      weights: mesh.getWeights(),
    };
  }
}

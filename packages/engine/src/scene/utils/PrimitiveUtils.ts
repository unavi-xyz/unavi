import { Document, GLTF, Primitive } from "@gltf-transform/core";
import { nanoid } from "nanoid";

import { AccessorUtils } from "./AccessorUtils";
import { MaterialUtils } from "./MaterialUtils";
import { Utils } from "./types";

type MaterialId = string;
type AccessorId = string;
export interface PrimitiveJSON {
  mode: GLTF.MeshPrimitiveMode;
  material: MaterialId | null;
  indices: AccessorId | null;
  attributes: { [key: string]: AccessorId };
  targets: PrimitiveTargetJSON[];
}

export interface PrimitiveTargetJSON {
  attributes: { [key: string]: AccessorId };
}

export class PrimitiveUtils implements Utils<Primitive, PrimitiveJSON> {
  #doc: Document;
  #accessor: AccessorUtils;
  #material: MaterialUtils;

  store = new Map<string, Primitive>();

  constructor(doc: Document, accessor: AccessorUtils, material: MaterialUtils) {
    this.#doc = doc;
    this.#accessor = accessor;
    this.#material = material;
  }

  getId(primitive: Primitive) {
    for (const [id, p] of this.store) {
      if (p === primitive) return id;
    }
  }

  create(json: Partial<PrimitiveJSON> = {}, id?: string) {
    const primitive = this.#doc.createPrimitive();
    this.applyJSON(primitive, json);

    const { id: primitiveId } = this.process(primitive, id);

    return { id: primitiveId, object: primitive };
  }

  process(primitive: Primitive, id?: string) {
    const primitiveId = id ?? nanoid();
    this.store.set(primitiveId, primitive);

    primitive.addEventListener("dispose", () => {
      this.store.delete(primitiveId);
    });

    return { id: primitiveId };
  }

  processChanges() {
    const changed: Primitive[] = [];

    // Add new primitives
    this.#doc
      .getRoot()
      .listMeshes()
      .forEach((mesh) => {
        mesh.listPrimitives().forEach((primitive) => {
          const primitiveId = this.getId(primitive);
          if (primitiveId) return;

          this.process(primitive);
          changed.push(primitive);
        });
      });

    return changed;
  }

  applyJSON(primitive: Primitive, json: Partial<PrimitiveJSON>) {
    if (json.mode !== undefined) primitive.setMode(json.mode);

    if (json.material !== undefined) {
      if (json.material === null) {
        primitive.setMaterial(null);
      } else {
        const material = this.#material.store.get(json.material);
        if (material === undefined) throw new Error("Material not found");
        primitive.setMaterial(material);
      }
    }

    if (json.indices !== undefined) {
      if (json.indices === null) {
        primitive.setIndices(null);
      } else {
        const accessor = this.#accessor.store.get(json.indices);
        if (accessor === undefined) throw new Error("Accessor not found");
        primitive.setIndices(accessor);
      }
    }

    if (json.attributes !== undefined) {
      Object.entries(json.attributes).forEach(([name, accessorId]) => {
        const accessor = this.#accessor.store.get(accessorId);
        if (accessor === undefined) throw new Error("Accessor not found");
        primitive.setAttribute(name, accessor);
      });
    }

    if (json.targets !== undefined) {
      json.targets.forEach((targetJSON) => {
        const target = this.#doc.createPrimitiveTarget();

        Object.entries(targetJSON.attributes).forEach(([name, accessorId]) => {
          const accessor = this.#accessor.store.get(accessorId);
          if (accessor === undefined) throw new Error("Accessor not found");
          target.setAttribute(name, accessor);
        });

        primitive.addTarget(target);
      });
    }
  }

  toJSON(primitive: Primitive): PrimitiveJSON {
    const material = primitive.getMaterial();
    const materialId = material ? this.#material.getId(material) : null;
    if (materialId === undefined) throw new Error("Material not found");

    const indices = primitive.getIndices();
    const indicesId = indices ? this.#accessor.getId(indices) : null;
    if (indicesId === undefined) throw new Error("Indices not found");

    const targets = primitive.listTargets().map((target) => {
      const targetJSON: PrimitiveTargetJSON = {
        attributes: {},
      };

      target.listAttributes().forEach((attribute) => {
        const accessorId = this.#accessor.getId(attribute);
        if (accessorId === undefined) throw new Error("Accessor not found");

        targetJSON.attributes[attribute.getName()] = accessorId;
      });

      return targetJSON;
    });

    const json: PrimitiveJSON = {
      mode: primitive.getMode(),
      material: materialId,
      indices: indicesId,
      attributes: {},
      targets,
    };

    primitive.listAttributes().forEach((attribute) => {
      const accessorId = this.#accessor.getId(attribute);
      if (accessorId === undefined) throw new Error("Accessor not found");

      json.attributes[attribute.getName()] = accessorId;
    });

    return json;
  }
}

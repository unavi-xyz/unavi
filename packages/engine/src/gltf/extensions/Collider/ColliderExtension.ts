import { Extension, ReaderContext, WriterContext } from "@gltf-transform/core";

import { Vec3 } from "../../../types";
import { Collider } from "./Collider";
import { EXTENSION_NAME } from "./constants";
import { ColliderType } from "./types";

type NodeColliderDef = {
  collider: number;
};

type ColliderDef = {
  type: ColliderType;
  size?: Vec3;
  radius?: number;
  height?: number;
  mesh?: number;
};

type ColliderExtensionDef = {
  colliders: ColliderDef[];
};

/**
 * @link https://github.com/omigroup/gltf-extensions/tree/main/extensions/2.0/OMI_collider
 */
export class ColliderExtension extends Extension {
  static override readonly EXTENSION_NAME = EXTENSION_NAME;
  override readonly extensionName = EXTENSION_NAME;

  createCollider(): Collider {
    return new Collider(this.document.getGraph());
  }

  read(context: ReaderContext): this {
    const jsonDoc = context.jsonDoc;

    if (!jsonDoc.json.extensions || !jsonDoc.json.extensions[EXTENSION_NAME]) return this;

    const rootDef = jsonDoc.json.extensions[EXTENSION_NAME] as ColliderExtensionDef;

    const colliderDefs = rootDef.colliders || ([] as ColliderDef[]);

    const colliders = colliderDefs.map((colliderDef) => {
      const collider = this.createCollider();
      collider.type = colliderDef.type;

      if (colliderDef.size !== undefined) collider.size = colliderDef.size;

      if (colliderDef.radius !== undefined) collider.radius = colliderDef.radius;

      if (colliderDef.height !== undefined) collider.height = colliderDef.height;

      if (colliderDef.mesh !== undefined) {
        const mesh = context.meshes[colliderDef.mesh];
        if (!mesh) throw new Error("Mesh not found");
        collider.mesh = mesh;
      }

      return collider;
    });

    const nodeDefs = jsonDoc.json.nodes || [];

    nodeDefs.forEach((nodeDef, nodeIndex) => {
      if (!nodeDef.extensions || !nodeDef.extensions[EXTENSION_NAME]) return;
      const colliderNodeDef = nodeDef.extensions[EXTENSION_NAME] as NodeColliderDef;

      const node = context.nodes[nodeIndex];
      if (!node) throw new Error("Node not found");

      const collider = colliders[colliderNodeDef.collider];
      if (!collider) throw new Error("Collider not found");

      node.setExtension(EXTENSION_NAME, collider);
    });

    return this;
  }

  write(context: WriterContext): this {
    const jsonDoc = context.jsonDoc;

    if (this.properties.size === 0) return this;

    const colliderDefs = [];
    const colliderIndexMap = new Map<Collider, number>();

    for (const property of this.properties) {
      const collider = property as Collider;
      const colliderDef = { type: collider.type } as ColliderDef;

      switch (collider.type) {
        case "box": {
          const size = collider.size;
          if (!size) throw new Error("Size not set");

          colliderDef.size = size;
          break;
        }

        case "sphere": {
          const radius = collider.radius;
          if (radius === null) throw new Error("Radius not set");

          colliderDef.radius = radius;
          break;
        }

        case "capsule":
        case "cylinder": {
          const radius = collider.radius;
          if (radius === null) throw new Error("Radius not set");

          const height = collider.height;
          if (height === null) throw new Error("Height not set");

          colliderDef.radius = radius;
          colliderDef.height = height;
          break;
        }

        case "trimesh": {
          const mesh = collider.mesh;
          if (!mesh) break;

          const meshIndex = context.meshIndexMap.get(mesh);
          if (meshIndex === undefined) throw new Error("Mesh not found");

          colliderDef.mesh = meshIndex;
          break;
        }
      }

      colliderDefs.push(colliderDef);
      colliderIndexMap.set(collider, colliderDefs.length - 1);
    }

    this.document
      .getRoot()
      .listNodes()
      .forEach((node) => {
        const collider = node.getExtension<Collider>(EXTENSION_NAME);

        if (collider) {
          const nodeIndex = context.nodeIndexMap.get(node);
          if (nodeIndex === undefined) throw new Error("Node index not found");

          const nodes = jsonDoc.json.nodes;
          if (!nodes) throw new Error("Nodes not found");

          const nodeDef = nodes[nodeIndex];
          if (!nodeDef) throw new Error("Node def not found");

          nodeDef.extensions = nodeDef.extensions || {};
          nodeDef.extensions[EXTENSION_NAME] = {
            collider: colliderIndexMap.get(collider),
          };
        }
      });

    jsonDoc.json.extensions = jsonDoc.json.extensions || {};
    jsonDoc.json.extensions[EXTENSION_NAME] = { colliders: colliderDefs };

    return this;
  }
}

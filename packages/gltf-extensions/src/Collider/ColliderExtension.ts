import { Extension, ReaderContext, WriterContext } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { Vec3 } from "../types";
import { Collider } from "./Collider";
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
 * Implementation of the {@link https://github.com/omigroup/gltf-extensions/tree/main/extensions/2.0/OMI_collider OMI_collider} extension.
 *
 * @group OMI_collider
 */
export class ColliderExtension extends Extension {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.Collider;
  override readonly extensionName = EXTENSION_NAME.Collider;

  createCollider(): Collider {
    return new Collider(this.document.getGraph());
  }

  read(context: ReaderContext) {
    if (!context.jsonDoc.json.extensions || !context.jsonDoc.json.extensions[this.extensionName])
      return this;

    const rootDef = context.jsonDoc.json.extensions[this.extensionName] as ColliderExtensionDef;

    const colliders = rootDef.colliders.map((colliderDef) => {
      const collider = this.createCollider();
      collider.setType(colliderDef.type);

      if (colliderDef.size !== undefined) collider.setSize(colliderDef.size);
      if (colliderDef.radius !== undefined) collider.setRadius(colliderDef.radius);
      if (colliderDef.height !== undefined) collider.setHeight(colliderDef.height);
      if (colliderDef.mesh !== undefined) {
        const mesh = context.meshes[colliderDef.mesh];
        if (!mesh) throw new Error("Mesh not found");
        collider.setMesh(mesh);
      }

      return collider;
    });

    const nodeDefs = context.jsonDoc.json.nodes ?? [];

    nodeDefs.forEach((nodeDef, nodeIndex) => {
      if (!nodeDef.extensions || !nodeDef.extensions[this.extensionName]) return;
      const colliderNodeDef = nodeDef.extensions[this.extensionName] as NodeColliderDef;

      const node = context.nodes[nodeIndex];
      if (!node) throw new Error("Node not found");

      const collider = colliders[colliderNodeDef.collider];
      if (!collider) throw new Error("Collider not found");

      node.setExtension(this.extensionName, collider);
    });

    return this;
  }

  write(context: WriterContext) {
    const jsonDoc = context.jsonDoc;

    if (this.properties.size === 0) return this;

    const colliderDefs = [];
    const colliderIndexMap = new Map<Collider, number>();

    for (const property of this.properties) {
      if (property instanceof Collider) {
        const colliderDef = { type: property.getType() } as ColliderDef;

        switch (property.getType()) {
          case "box": {
            const size = property.getSize();
            if (!size) throw new Error("Size not set");

            colliderDef.size = size;
            break;
          }

          case "sphere": {
            const radius = property.getRadius();
            if (radius === null) throw new Error("Radius not set");

            colliderDef.radius = radius;
            break;
          }

          case "capsule":
          case "cylinder": {
            const radius = property.getRadius();
            if (radius === null) throw new Error("Radius not set");

            const height = property.getHeight();
            if (height === null) throw new Error("Height not set");

            colliderDef.radius = radius;
            colliderDef.height = height;
            break;
          }

          case "trimesh": {
            const mesh = property.getMesh();
            if (!mesh) break;

            const meshIndex = context.meshIndexMap.get(mesh);
            if (meshIndex === undefined) throw new Error("Mesh not found");

            colliderDef.mesh = meshIndex;
            break;
          }
        }

        colliderDefs.push(colliderDef);
        colliderIndexMap.set(property, colliderDefs.length - 1);
      }
    }

    this.document
      .getRoot()
      .listNodes()
      .forEach((node) => {
        const collider = node.getExtension<Collider>(this.extensionName);

        if (collider) {
          const nodeIndex = context.nodeIndexMap.get(node);
          if (nodeIndex === undefined) throw new Error("Node index not found");

          const nodes = jsonDoc.json.nodes;
          if (!nodes) throw new Error("Nodes not found");

          const nodeDef = nodes[nodeIndex];
          if (!nodeDef) throw new Error("Node def not found");

          nodeDef.extensions = nodeDef.extensions ?? {};
          nodeDef.extensions[this.extensionName] = {
            collider: colliderIndexMap.get(collider),
          };
        }
      });

    if (colliderDefs.length > 0) {
      const rootDef: ColliderExtensionDef = { colliders: colliderDefs };

      if (!jsonDoc.json.extensions) jsonDoc.json.extensions = {};
      jsonDoc.json.extensions[this.extensionName] = rootDef;
    }

    return this;
  }
}

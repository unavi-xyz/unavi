import { Document, Node } from "@gltf-transform/core";
import { nanoid } from "nanoid";

import { Collider } from "../../gltf/extensions/Collider/Collider";
import { ColliderExtension } from "../../gltf/extensions/Collider/ColliderExtension";
import { ColliderType } from "../../gltf/extensions/Collider/types";
import { Vec3, Vec4 } from "../../types";
import { Attribute } from "./Attribute";
import { Meshes } from "./Meshes";

export type ColliderJSON = {
  type: ColliderType;
  size: Vec3 | null;
  radius: number | null;
  height: number | null;
  mesh: string | null;
};

type NodeExtensionsJSON = {
  [ColliderExtension.EXTENSION_NAME]: ColliderJSON | null;
};

type MeshId = string;
type SkinId = string;
type NodeId = string;

export type NodeJSON = {
  name: string;
  translation: Vec3;
  rotation: Vec4;
  scale: Vec3;
  mesh: MeshId | null;
  skin: SkinId | null;
  children: NodeId[];
  extensions: NodeExtensionsJSON;
};

export class Nodes extends Attribute<Node, NodeJSON> {
  #doc: Document;
  #mesh: Meshes;

  store = new Map<string, Node>();

  constructor(doc: Document, mesh: Meshes) {
    super();

    this.#doc = doc;
    this.#mesh = mesh;
  }

  getId(node: Node) {
    for (const [id, n] of this.store) {
      if (n === node) return id;
    }
  }

  getParent(node: Node) {
    for (const [id, n] of this.store) {
      if (n.listChildren().includes(node)) return id;
    }
  }

  create(json: Partial<NodeJSON> = {}, id?: string) {
    const node = this.#doc.createNode();
    this.applyJSON(node, json);

    const { id: nodeId } = this.process(node, id);

    this.emitCreate(nodeId);

    return { id: nodeId, object: node };
  }

  process(node: Node, id?: string) {
    const nodeId = id ?? nanoid();
    this.store.set(nodeId, node);

    node.addEventListener("dispose", () => {
      this.store.delete(nodeId);
    });

    return { id: nodeId };
  }

  processChanges() {
    const changed: Node[] = [];

    // Add new nodes
    this.#doc
      .getRoot()
      .listNodes()
      .forEach((node) => {
        const nodeId = this.getId(node);
        if (nodeId) return;

        this.process(node);
        changed.push(node);
      });

    return changed;
  }

  applyJSON(node: Node, json: Partial<NodeJSON>) {
    if (json.name !== undefined) node.setName(json.name);
    if (json.translation) node.setTranslation(json.translation);
    if (json.rotation) node.setRotation(json.rotation);
    if (json.scale) node.setScale(json.scale);

    if (json.mesh !== undefined) {
      if (json.mesh === null) {
        node.setMesh(null);
      } else {
        const mesh = this.#mesh.store.get(json.mesh);
        if (!mesh) throw new Error("Mesh not found");
        node.setMesh(mesh);
      }
    }

    if (json.children) {
      for (const childId of json.children) {
        const child = this.store.get(childId);
        if (!child) continue;

        node.addChild(child);
      }
    }

    if (json.extensions) {
      const colliderJSON = json.extensions[ColliderExtension.EXTENSION_NAME];

      if (colliderJSON) {
        const collider = node.getExtension<Collider>(ColliderExtension.EXTENSION_NAME);

        if (collider) {
          collider.type = colliderJSON.type;
          collider.size = colliderJSON.size ?? null;
          collider.height = colliderJSON.height ?? null;
          collider.radius = colliderJSON.radius ?? null;

          if (colliderJSON.mesh) {
            const mesh = this.#mesh.store.get(colliderJSON.mesh);
            if (!mesh) throw new Error("Mesh not found");
            collider.mesh = mesh;
          }
        }
      }
    }
  }

  toJSON(node: Node): NodeJSON {
    const mesh = node.getMesh();
    const meshId = mesh ? this.#mesh.getId(mesh) : null;
    if (meshId === undefined) throw new Error("Mesh not found");

    const childrenIds = node.listChildren().map((child) => {
      for (const [id, node] of this.store) {
        if (node === child) return id;
      }
      throw new Error("Child not found");
    });

    const extensions: NodeExtensionsJSON = {
      [ColliderExtension.EXTENSION_NAME]: null,
    };

    const collider = node.getExtension<Collider>(ColliderExtension.EXTENSION_NAME);

    if (collider) {
      const mesh = collider.mesh;
      const meshId = mesh ? this.#mesh.getId(mesh) : null;
      if (meshId === undefined) throw new Error("Mesh not found");

      extensions[ColliderExtension.EXTENSION_NAME] = {
        type: collider.type,
        size: collider.size,
        height: collider.height,
        radius: collider.radius,
        mesh: meshId,
      };
    }

    return {
      name: node.getName(),
      translation: node.getTranslation(),
      rotation: node.getRotation(),
      scale: node.getScale(),
      mesh: meshId,
      skin: null,
      children: childrenIds,
      extensions,
    };
  }
}

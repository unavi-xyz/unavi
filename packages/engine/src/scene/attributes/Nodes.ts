import { Node } from "@gltf-transform/core";
import { nanoid } from "nanoid";

import { SpawnPoint, SpawnPointExtension } from "../../gltf";
import { Collider } from "../../gltf/extensions/Collider/Collider";
import { ColliderExtension } from "../../gltf/extensions/Collider/ColliderExtension";
import { ColliderType } from "../../gltf/extensions/Collider/types";
import { Vec3, Vec4 } from "../../types";
import { Scene } from "../Scene";
import { Attribute } from "./Attribute";

export type ColliderJSON = {
  type: ColliderType;
  size: Vec3 | null;
  radius: number | null;
  height: number | null;
  mesh: string | null;
};

export type SpawnPointJSON = {
  title: string;
};

type NodeExtensionsJSON = {
  [ColliderExtension.EXTENSION_NAME]: ColliderJSON | null;
  [SpawnPointExtension.EXTENSION_NAME]: SpawnPointJSON | null;
};

type MeshId = string;
type SkinId = string;
type NodeId = string;

export type NodeExtras = {
  scripts?: Array<{ id: string; name: string }>;
};

export type NodeJSON = {
  name: string;
  translation: Vec3;
  rotation: Vec4;
  scale: Vec3;
  mesh: MeshId | null;
  skin: SkinId | null;
  children: NodeId[];
  extensions: NodeExtensionsJSON;
  extras: NodeExtras;
};

export class Nodes extends Attribute<Node, NodeJSON> {
  #scene: Scene;

  store = new Map<string, Node>();

  constructor(scene: Scene) {
    super();

    this.#scene = scene;
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
    const node = this.#scene.doc.createNode();
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
    const nodes = this.#scene.doc.getRoot().listNodes();

    // Sort by depth
    function getDepth(node: Node): number {
      const parents = node.listParents().filter((parent) => parent instanceof Node);
      if (parents.length === 0) return 0;

      const parent = parents[0] as Node;
      return getDepth(parent) + 1;
    }

    const sortedNodes = nodes
      .map((node) => {
        const depth = getDepth(node);
        if (depth === undefined) throw new Error("Depth not found");
        return { node, depth };
      })
      .sort((a, b) => b.depth - a.depth);

    // Add new nodes
    const changed: Node[] = [];

    sortedNodes.forEach(({ node }) => {
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
        const mesh = this.#scene.mesh.store.get(json.mesh);
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

      if (!colliderJSON) {
        node.setExtension(ColliderExtension.EXTENSION_NAME, null);
      } else {
        const collider =
          node.getExtension<Collider>(ColliderExtension.EXTENSION_NAME) ??
          this.#scene.extensions.collider.createCollider();

        collider.type = colliderJSON.type;
        collider.size = colliderJSON.size;
        collider.height = colliderJSON.height;
        collider.radius = colliderJSON.radius;

        if (colliderJSON.mesh) {
          const mesh = this.#scene.mesh.store.get(colliderJSON.mesh);
          if (!mesh) throw new Error("Mesh not found");
          collider.mesh = mesh;
        }

        node.setExtension(ColliderExtension.EXTENSION_NAME, collider);
      }

      const spawnPointJSON = json.extensions[SpawnPointExtension.EXTENSION_NAME];

      if (!spawnPointJSON) {
        node.setExtension(SpawnPointExtension.EXTENSION_NAME, null);
      } else {
        const spawnPoint =
          node.getExtension<SpawnPoint>(SpawnPointExtension.EXTENSION_NAME) ??
          this.#scene.extensions.spawn.createSpawnPoint();

        spawnPoint.title = spawnPointJSON.title;

        node.setExtension(SpawnPointExtension.EXTENSION_NAME, spawnPoint);
      }
    }

    if (json.extras) node.setExtras(json.extras);
  }

  toJSON(node: Node): NodeJSON {
    const mesh = node.getMesh();
    const meshId = mesh ? this.#scene.mesh.getId(mesh) : null;
    if (meshId === undefined) throw new Error("Mesh not found");

    const childrenIds = node.listChildren().map((child) => {
      for (const [id, node] of this.store) {
        if (node === child) return id;
      }
      throw new Error("Child not found");
    });

    const extensions: NodeExtensionsJSON = {
      [ColliderExtension.EXTENSION_NAME]: null,
      [SpawnPointExtension.EXTENSION_NAME]: null,
    };

    const collider = node.getExtension<Collider>(ColliderExtension.EXTENSION_NAME);

    if (collider) {
      const mesh = collider.mesh;
      const meshId = mesh ? this.#scene.mesh.getId(mesh) : null;
      if (meshId === undefined) throw new Error("Mesh not found");

      extensions[ColliderExtension.EXTENSION_NAME] = {
        type: collider.type,
        size: collider.size,
        height: collider.height,
        radius: collider.radius,
        mesh: meshId,
      };
    }

    const spawnPoint = node.getExtension<SpawnPoint>(SpawnPointExtension.EXTENSION_NAME);

    if (spawnPoint) {
      extensions[SpawnPointExtension.EXTENSION_NAME] = {
        title: spawnPoint.title,
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
      extras: node.getExtras() as NodeExtras,
    };
  }
}

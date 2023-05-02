import { Node } from "@gltf-transform/core";
import {
  Avatar,
  AvatarExtension,
  Collider,
  ColliderExtension,
  ColliderType,
  SpawnPoint,
  SpawnPointExtension,
} from "@unavi/gltf-extensions";
import { nanoid } from "nanoid";

import { Vec3, Vec4 } from "../../types";
import { Scene } from "../Scene";
import { Attribute } from "./Attribute";

export type AvatarJSON = {
  name: string;
  equippable: boolean;
  uri: string;
};

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
  [AvatarExtension.EXTENSION_NAME]: AvatarJSON | null;
  [ColliderExtension.EXTENSION_NAME]: ColliderJSON | null;
  [SpawnPointExtension.EXTENSION_NAME]: SpawnPointJSON | null;
};

type MeshId = string;
type SkinId = string;
type NodeId = string;

export type NodeExtras = {
  euler?: Vec3;
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

/**
 * Stores and manages nodes within a {@link Scene}.
 *
 * @group Scene
 */
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

    // Add to scene
    const scene = this.#scene.doc.getRoot().listScenes()[0];
    scene?.addChild(node);

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
        return { depth, node };
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
    if (json.translation !== undefined) node.setTranslation(json.translation);
    if (json.rotation !== undefined) node.setRotation(json.rotation);
    if (json.scale !== undefined) node.setScale(json.scale);

    if (json.mesh !== undefined) {
      if (json.mesh === null) {
        node.setMesh(null);
      } else {
        const mesh = this.#scene.mesh.store.get(json.mesh);
        if (!mesh) throw new Error("Mesh not found");
        node.setMesh(mesh);
      }
    }

    if (json.skin !== undefined) {
      if (json.skin === null) {
        node.setSkin(null);
      } else {
        const skin = this.#scene.skin.store.get(json.skin);
        if (skin) node.setSkin(skin);
      }
    }

    if (json.children !== undefined) {
      for (const childId of json.children) {
        const child = this.store.get(childId);
        if (!child) continue;

        node.addChild(child);
      }

      // Remove children that are not in the json
      const children = node.listChildren();

      for (const child of children) {
        const childId = this.getId(child);
        if (!childId) continue;

        if (!json.children.includes(childId)) {
          node.removeChild(child);
        }
      }
    }

    if (json.extensions !== undefined) {
      const avatarJSON = json.extensions[AvatarExtension.EXTENSION_NAME];
      if (!avatarJSON) {
        node.setExtension(AvatarExtension.EXTENSION_NAME, null);
      } else {
        let avatar = node.getExtension<Avatar>(AvatarExtension.EXTENSION_NAME);

        if (!avatar) {
          avatar = this.#scene.extensions.avatar.createAvatar();
          node.setExtension(AvatarExtension.EXTENSION_NAME, avatar);
        }

        avatar.setName(avatarJSON.name);
        avatar.setEquippable(avatarJSON.equippable);
        avatar.setURI(avatarJSON.uri);
      }

      const colliderJSON = json.extensions[ColliderExtension.EXTENSION_NAME];
      if (!colliderJSON) {
        node.setExtension(ColliderExtension.EXTENSION_NAME, null);
      } else {
        const collider =
          node.getExtension<Collider>(ColliderExtension.EXTENSION_NAME) ??
          this.#scene.extensions.collider.createCollider();

        collider.setType(colliderJSON.type);
        collider.setSize(colliderJSON.size);
        collider.setHeight(colliderJSON.height);
        collider.setRadius(colliderJSON.radius);

        if (colliderJSON.mesh) {
          const mesh = this.#scene.mesh.store.get(colliderJSON.mesh);
          if (!mesh) throw new Error("Mesh not found");
          collider.setMesh(mesh);
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

        spawnPoint.setTitle(spawnPointJSON.title);

        node.setExtension(SpawnPointExtension.EXTENSION_NAME, spawnPoint);
      }
    }

    if (json.extras) node.setExtras(json.extras);
  }

  toJSON(node: Node): NodeJSON {
    const mesh = node.getMesh();
    const meshId = mesh ? this.#scene.mesh.getId(mesh) : null;
    if (meshId === undefined) throw new Error("Mesh not found");

    const skin = node.getSkin();
    const skinId = skin ? this.#scene.skin.getId(skin) : null;
    if (skinId === undefined) throw new Error("Skin not found");

    const childrenIds = node.listChildren().map((child) => {
      for (const [id, node] of this.store) {
        if (node === child) return id;
      }
      throw new Error("Child not found");
    });

    const extensions: NodeExtensionsJSON = {
      [AvatarExtension.EXTENSION_NAME]: null,
      [ColliderExtension.EXTENSION_NAME]: null,
      [SpawnPointExtension.EXTENSION_NAME]: null,
    };

    const avatar = node.getExtension<Avatar>(AvatarExtension.EXTENSION_NAME);
    if (avatar) {
      extensions[AvatarExtension.EXTENSION_NAME] = {
        equippable: avatar.getEquippable(),
        name: avatar.getName(),
        uri: avatar.getURI(),
      };
    }

    const collider = node.getExtension<Collider>(ColliderExtension.EXTENSION_NAME);
    if (collider) {
      const mesh = collider.getMesh();
      const meshId = mesh ? this.#scene.mesh.getId(mesh) : null;
      if (meshId === undefined) throw new Error("Mesh not found");

      extensions[ColliderExtension.EXTENSION_NAME] = {
        height: collider.getHeight(),
        mesh: meshId,
        radius: collider.getRadius(),
        size: collider.getSize(),
        type: collider.getType(),
      };
    }

    const spawnPoint = node.getExtension<SpawnPoint>(SpawnPointExtension.EXTENSION_NAME);
    if (spawnPoint) {
      extensions[SpawnPointExtension.EXTENSION_NAME] = {
        title: spawnPoint.getTitle(),
      };
    }

    return {
      children: childrenIds,
      extensions,
      extras: node.getExtras() as NodeExtras,
      mesh: meshId,
      name: node.getName(),
      rotation: node.getRotation(),
      scale: node.getScale(),
      skin: skinId,
      translation: node.getTranslation(),
    };
  }
}

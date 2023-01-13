import { Collider, ColliderDesc, RigidBodyDesc, World } from "@dimforge/rapier3d";

import { NodeJSON } from "../scene";
import { SceneMessage } from "../scene/messages";
import { Scene } from "../scene/Scene";
import { COLLISION_GROUP } from "./groups";

export class PhysicsScene extends Scene {
  #world: World;

  colliders = new Map<string, Collider>();

  constructor(world: World) {
    super();

    this.#world = world;
  }

  onmessage = ({ subject, data }: SceneMessage) => {
    switch (subject) {
      case "create_node": {
        if (data.json.mesh) data.json.mesh = null;

        const { object: node } = this.node.create(data.json, data.id);

        node.addEventListener("dispose", () => {
          this.#removeNodeCollider(data.id);
        });

        node.addEventListener("change", (e) => {
          const attribute = e.attribute as keyof NodeJSON;

          if (attribute === "translation" || attribute === "rotation") {
            this.#updateNodeTransform(data.id);
          }
        });

        this.#updateNode(data.id, data.json);
        this.#updateNodeTransform(data.id);
        break;
      }

      case "change_node": {
        const node = this.node.store.get(data.id);
        if (!node) throw new Error("Node not found");

        if (data.json.mesh) delete data.json.mesh;

        this.node.applyJSON(node, data.json);

        this.#updateNode(data.id, data.json);
        break;
      }

      case "dispose_node": {
        const node = this.node.store.get(data);
        if (node) node.dispose();
        break;
      }
    }
  };

  #updateNode(nodeId: string, json: Partial<NodeJSON>) {
    const colliderJSON = json.extensions?.OMI_collider;

    if (colliderJSON) {
      // Remove existing collider
      this.#removeNodeCollider(nodeId);

      // Create new collider
      let colliderDesc: ColliderDesc | undefined;

      switch (colliderJSON.type) {
        case "box": {
          const size = colliderJSON.size ?? [1, 1, 1];
          colliderDesc = ColliderDesc.cuboid(size[0] / 2, size[1] / 2, size[2] / 2);
          break;
        }

        case "sphere": {
          const radius = colliderJSON.radius ?? 0.5;
          colliderDesc = ColliderDesc.ball(radius);
          break;
        }

        case "cylinder": {
          const height = colliderJSON.height ?? 1;
          const radius = colliderJSON.radius ?? 0.5;
          colliderDesc = ColliderDesc.cylinder(height / 2, radius);
          break;
        }
      }

      if (!colliderDesc) return;

      colliderDesc.setCollisionGroups(COLLISION_GROUP.static);

      const rigidBodyDesc = RigidBodyDesc.fixed();
      const rigidBody = this.#world.createRigidBody(rigidBodyDesc);
      const collider = this.#world.createCollider(colliderDesc, rigidBody);

      this.colliders.set(nodeId, collider);
    }
  }

  #removeNodeCollider(nodeId: string) {
    const collider = this.colliders.get(nodeId);
    if (collider) {
      const rigidBody = collider.parent();
      if (rigidBody) this.#world.removeRigidBody(rigidBody);
      this.#world.removeCollider(collider, true);
      this.colliders.delete(nodeId);
    }
  }

  #updateNodeTransform(nodeId: string) {
    const node = this.node.store.get(nodeId);
    if (!node) throw new Error("Node not found");

    const worldTranslation = node.getWorldTranslation();
    const worldRotation = node.getWorldRotation();

    const collider = this.colliders.get(nodeId);
    const rigidBody = collider?.parent();

    if (rigidBody) {
      rigidBody.setTranslation(
        {
          x: worldTranslation[0],
          y: worldTranslation[1],
          z: worldTranslation[2],
        },
        true
      );

      rigidBody.setRotation(
        {
          x: worldRotation[0],
          y: worldRotation[1],
          z: worldRotation[2],
          w: worldRotation[3],
        },
        true
      );
    }

    // Update children
    for (const child of node.listChildren()) {
      const childId = this.node.getId(child);
      if (childId) this.#updateNodeTransform(childId);
    }
  }
}

import { Collider, ColliderDesc, RigidBodyDesc, TriMesh, World } from "@dimforge/rapier3d";
import { Node } from "@gltf-transform/core";

import { NodeJSON } from "../scene";
import { SceneMessage } from "../scene/messages";
import { Scene } from "../scene/Scene";
import { Vec3 } from "../types";
import { COLLISION_GROUP } from "./groups";

/**
 * Instance of a {@link Scene} for the physics thread.
 * This class is responsible for creating and disposing physics objects.
 */
export class PhysicsScene extends Scene {
  #world: World;

  colliders = new Map<string, Collider[]>();
  colliderScales = new Map<string, Vec3[]>();

  constructor(world: World) {
    super();

    this.#world = world;
  }

  onmessage = ({ subject, data }: SceneMessage) => {
    switch (subject) {
      case "create_buffer": {
        this.buffer.create(data.json, data.id);
        break;
      }

      case "dispose_buffer": {
        const buffer = this.buffer.store.get(data);
        if (!buffer) throw new Error("Buffer not found");
        buffer.dispose();
        break;
      }

      case "create_accessor": {
        this.accessor.create(data.json, data.id);
        break;
      }

      case "dispose_accessor": {
        const accessor = this.accessor.store.get(data);
        if (!accessor) throw new Error("Accessor not found");
        accessor.dispose();
        break;
      }

      case "create_primitive": {
        if (data.json.material) data.json.material = null;
        this.primitive.create(data.json, data.id);
        break;
      }

      case "change_primitive": {
        if (data.json.material) data.json.material = null;

        const primitive = this.primitive.store.get(data.id);
        if (!primitive) throw new Error(`Primitive not found: ${data.id}`);

        this.primitive.applyJSON(primitive, data.json);
        break;
      }

      case "dispose_primitive": {
        const primitive = this.primitive.store.get(data);
        if (!primitive) throw new Error(`Primitive not found: ${data}`);

        primitive.dispose();
        break;
      }

      case "create_mesh": {
        this.mesh.create(data.json, data.id);
        break;
      }

      case "change_mesh": {
        const mesh = this.mesh.store.get(data.id);
        if (!mesh) throw new Error("Mesh not found");

        this.mesh.applyJSON(mesh, data.json);

        mesh.listParents().forEach((parent) => {
          if (parent instanceof Node) {
            const nodeId = this.node.getId(parent);
            if (!nodeId) throw new Error("Node not found");

            const json = this.node.toJSON(parent);

            this.#updateNode(nodeId, { extensions: json.extensions });
          }
        });
        break;
      }

      case "dispose_mesh": {
        const mesh = this.mesh.store.get(data);
        if (!mesh) throw new Error("Mesh not found");
        mesh.dispose();
        break;
      }

      case "create_node": {
        const { object: node } = this.node.create(data.json, data.id);

        node.addEventListener("dispose", () => {
          this.#removeNodeCollider(data.id);
        });

        node.addEventListener("change", (e) => {
          const attribute = e.attribute as keyof NodeJSON;

          if (attribute === "translation" || attribute === "rotation" || attribute === "scale") {
            this.#updateNodeTransform(data.id);
          }
        });

        this.#updateNodeCollider(data.id);
        this.#updateNodeTransform(data.id);
        break;
      }

      case "change_node": {
        const node = this.node.store.get(data.id);
        if (!node) throw new Error("Node not found");

        this.node.applyJSON(node, data.json);

        this.#updateNode(data.id, data.json);
        break;
      }

      case "dispose_node": {
        const node = this.node.store.get(data);
        if (!node) throw new Error("Node not found");
        node.dispose();
        break;
      }
    }
  };

  #updateNode(nodeId: string, json: Partial<NodeJSON>) {
    const colliderJSON = json.extensions?.OMI_collider;

    if (colliderJSON !== undefined) {
      // Remove existing collider
      this.#removeNodeCollider(nodeId);

      // Create new collider
      if (colliderJSON) {
        const colliderDescs: ColliderDesc[] = [];

        switch (colliderJSON.type) {
          case "box": {
            const size = colliderJSON.size ?? [1, 1, 1];
            colliderDescs.push(ColliderDesc.cuboid(size[0] / 2, size[1] / 2, size[2] / 2));
            break;
          }

          case "sphere": {
            const radius = colliderJSON.radius ?? 0.5;
            colliderDescs.push(ColliderDesc.ball(radius));
            break;
          }

          case "cylinder": {
            const height = colliderJSON.height ?? 1;
            const radius = colliderJSON.radius ?? 0.5;
            colliderDescs.push(ColliderDesc.cylinder(height / 2, radius));
            break;
          }

          case "trimesh": {
            if (!colliderJSON.mesh) break;

            const mesh = this.mesh.store.get(colliderJSON.mesh);
            if (!mesh) throw new Error("Mesh not found");

            const vertices = mesh.listPrimitives().map((primitive) => {
              const attribute = primitive.getAttribute("POSITION");
              if (!attribute) return [];

              const array = attribute.getArray();
              if (!array) return [];

              return Array.from(array);
            });

            const indices = mesh.listPrimitives().map((primitive) => {
              const indicesAttribute = primitive.getIndices();
              if (!indicesAttribute) return [];

              const array = indicesAttribute.getArray();
              if (!array) return [];

              return Array.from(array);
            });

            vertices.forEach((vertices, i) => {
              const index = indices[i];
              if (!index) throw new Error("Indices not found");

              colliderDescs.push(
                ColliderDesc.trimesh(Float32Array.from(vertices), Uint32Array.from(index))
              );
            });
          }
        }

        if (colliderDescs.length > 0) {
          const rigidBodyDesc = RigidBodyDesc.kinematicPositionBased();
          const rigidBody = this.#world.createRigidBody(rigidBodyDesc);

          const colliders = colliderDescs.map((colliderDesc) => {
            colliderDesc.setCollisionGroups(COLLISION_GROUP.static);
            return this.#world.createCollider(colliderDesc, rigidBody);
          });

          this.colliders.set(nodeId, colliders);
        }
      }
    }
    this.#updateNodeTransform(nodeId);
  }

  #removeNodeCollider(nodeId: string) {
    const colliders = this.colliders.get(nodeId);
    if (colliders) {
      // Remove rigid body
      const collider = colliders[0];
      const rigidBody = collider?.parent();
      if (rigidBody) this.#world.removeRigidBody(rigidBody);

      // Remove colliders
      colliders.forEach((collider) => this.#world.removeCollider(collider, true));
      this.colliders.delete(nodeId);
      this.colliderScales.delete(nodeId);
    }
  }

  #updateNodeCollider(nodeId: string) {
    const node = this.node.store.get(nodeId);
    if (!node) throw new Error("Node not found");

    const json = this.node.toJSON(node);

    this.#updateNode(nodeId, { extensions: json.extensions });
  }

  #updateNodeTransform(nodeId: string) {
    const node = this.node.store.get(nodeId);
    if (!node) throw new Error("Node not found");

    const worldTranslation = node.getWorldTranslation();
    const worldRotation = node.getWorldRotation();
    const worldScale = node.getWorldScale();

    const colliders = this.colliders.get(nodeId);

    if (colliders) {
      colliders.forEach((collider, i) => {
        if (collider.shape instanceof TriMesh) {
          const prevScales = this.colliderScales.get(nodeId);
          const prevScale: Vec3 = prevScales ? prevScales[i] ?? [1, 1, 1] : [1, 1, 1];

          const sameScale = prevScale.every((value, index) => value === worldScale[index]);

          if (!sameScale) {
            // Apply scale to vertices
            const vertices = collider.shape.vertices;

            for (let i = 0; i < vertices.length; i += 3) {
              vertices[i] *= worldScale[0] / prevScale[0];
              vertices[i + 1] *= worldScale[1] / prevScale[1];
              vertices[i + 2] *= worldScale[2] / prevScale[2];
            }

            // Create new collider
            const newColliderDesc = ColliderDesc.trimesh(vertices, collider.shape.indices);
            newColliderDesc.setCollisionGroups(COLLISION_GROUP.static);

            const rigidBody = collider.parent();
            if (!rigidBody) throw new Error("RigidBody not found");

            // Update colliders array
            const newCollider = this.#world.createCollider(newColliderDesc, rigidBody);
            const newColliders = [...(colliders ?? [])];
            newColliders[i] = newCollider;
            this.colliders.set(nodeId, newColliders);

            // Remove old collider
            this.#world.removeCollider(collider, true);

            // Update scale array
            const newScales = [...(prevScales ?? [])];
            newScales[i] = worldScale;
            this.colliderScales.set(nodeId, newScales);
          }
        }

        const rigidBody = collider.parent();

        if (rigidBody) {
          rigidBody.setNextKinematicTranslation({
            x: worldTranslation[0],
            y: worldTranslation[1],
            z: worldTranslation[2],
          });

          rigidBody.setNextKinematicRotation({
            x: worldRotation[0],
            y: worldRotation[1],
            z: worldRotation[2],
            w: worldRotation[3],
          });
        }
      });
    }

    // Update children
    for (const child of node.listChildren()) {
      const childId = this.node.getId(child);
      if (childId) this.#updateNodeTransform(childId);
    }
  }
}

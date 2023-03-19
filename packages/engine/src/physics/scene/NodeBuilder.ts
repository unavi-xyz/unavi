import { Collider, ColliderDesc, RigidBody, RigidBodyDesc, TriMesh } from "@dimforge/rapier3d";
import { Node } from "@gltf-transform/core";
import { Collider as ColliderExt } from "@wired-labs/gltf-extensions";

import { NodeJSON } from "../../scene";
import { Vec3 } from "../../types";
import { subscribe } from "../../utils/subscribe";
import { COLLISION_GROUP } from "../groups";
import { PhysicsScene } from "./PhysicsScene";

/**
 * @internal
 * Handles the conversion of nodes to physics objects.
 */
export class NodeBuilder {
  #scene: PhysicsScene;

  rigidBodies = new Map<Node, RigidBody>();
  primitiveColliders = new Map<Node, Collider[]>();
  previousChildren = new Map<Node, Node[]>();

  constructor(scene: PhysicsScene) {
    this.#scene = scene;
  }

  add(json: Partial<NodeJSON>, id: string) {
    const { object: node } = this.#scene.node.create(json, id);

    subscribe(node, "Children", (children) => {
      const previousChildren = this.previousChildren.get(node) ?? [];
      const removedChildren = previousChildren.filter((child) => !children.includes(child));

      removedChildren.forEach((child) => {
        setTimeout(() => {
          this.#updateWorldTransform(child);
        });
      });

      this.previousChildren.set(node, children);

      return subscribe(node, "Translation", () =>
        subscribe(node, "Rotation", () =>
          subscribe(node, "Scale", () => {
            // Was facing issue where child nodes wouldn't have a parent
            // But the parent would have the node as a child
            // idk, this fixes it
            const timeout = setTimeout(() => {
              this.#updateWorldTransform(node);
            });

            return () => {
              clearTimeout(timeout);
            };
          })
        )
      );
    });

    subscribe(node, "Extensions", (extensions) => {
      const cleanup: (() => void)[] = [];

      const colliderExtension = extensions.find(
        (ext): ext is ColliderExt => ext instanceof ColliderExt
      );
      if (!colliderExtension) return;

      // Create new rigid body
      {
        const rigidBodyDesc = RigidBodyDesc.kinematicPositionBased();
        const rigidBody = this.#scene.world.createRigidBody(rigidBodyDesc);
        this.rigidBodies.set(node, rigidBody);

        this.#updateWorldTransform(node);
      }

      // Create colliders
      cleanup.push(
        subscribe(colliderExtension, "Type", (type) => {
          switch (type) {
            case "box": {
              return subscribe(colliderExtension, "Size", (size) => {
                const usedSize = size ?? ([1, 1, 1] as Vec3);

                const colliderDesc = ColliderDesc.cuboid(
                  usedSize[0] / 2,
                  usedSize[1] / 2,
                  usedSize[2] / 2
                );
                colliderDesc.setCollisionGroups(COLLISION_GROUP.static);

                const rigidBody = this.rigidBodies.get(node);
                if (!rigidBody) return;

                const collider = this.#scene.world.createCollider(colliderDesc, rigidBody);

                return () => {
                  this.#scene.world.removeCollider(collider, true);
                };
              });
            }

            case "sphere": {
              return subscribe(colliderExtension, "Radius", (radius) => {
                const usedRadius = radius ?? 0.5;

                const colliderDesc = ColliderDesc.ball(usedRadius);
                colliderDesc.setCollisionGroups(COLLISION_GROUP.static);

                const rigidBody = this.rigidBodies.get(node);
                if (!rigidBody) return;

                const collider = this.#scene.world.createCollider(colliderDesc, rigidBody);

                return () => {
                  this.#scene.world.removeCollider(collider, true);
                };
              });
            }

            case "cylinder": {
              return subscribe(colliderExtension, "Height", (height) => {
                return subscribe(colliderExtension, "Radius", (radius) => {
                  const usedHeight = height ?? 1;
                  const usedRadius = radius ?? 0.5;

                  const colliderDesc = ColliderDesc.cylinder(usedHeight / 2, usedRadius);
                  colliderDesc.setCollisionGroups(COLLISION_GROUP.static);

                  const rigidBody = this.rigidBodies.get(node);
                  if (!rigidBody) return;

                  const collider = this.#scene.world.createCollider(colliderDesc, rigidBody);

                  return () => {
                    this.#scene.world.removeCollider(collider, true);
                  };
                });
              });
            }

            case "trimesh": {
              return subscribe(colliderExtension, "Mesh", (mesh) => {
                if (!mesh) return;

                return subscribe(mesh, "Primitives", (primitives) => {
                  const currentColliders = this.primitiveColliders.get(node);

                  // Initialize colliders
                  if (!currentColliders) {
                    const vertices = primitives.map((primitive) => {
                      const attribute = primitive.getAttribute("POSITION");
                      const array = attribute?.getArray();
                      return Float32Array.from(array ?? []);
                    });

                    const indices = primitives.map((primitive) => {
                      const indicesAttribute = primitive.getIndices();
                      const array = indicesAttribute?.getArray();
                      return Uint32Array.from(array ?? []);
                    });

                    const colliders: Collider[] = [];

                    vertices.forEach((vertex, i) => {
                      const rigidBody = this.rigidBodies.get(node);
                      if (!rigidBody) return;

                      const index = indices[i];
                      if (!index) return;

                      if (vertex.length === 0 || index.length === 0) return;

                      // Create new collider
                      const colliderDesc = ColliderDesc.trimesh(vertex, index);
                      colliderDesc.setCollisionGroups(COLLISION_GROUP.static);

                      colliders.push(this.#scene.world.createCollider(colliderDesc, rigidBody));
                    });

                    this.primitiveColliders.set(node, colliders);
                  }

                  this.#updatePrimitiveColliders(node);

                  return () => {
                    const primitiveColliders = this.primitiveColliders.get(node);

                    primitiveColliders?.forEach((collider) => {
                      this.#scene.world.removeCollider(collider, true);
                    });

                    this.primitiveColliders.delete(node);
                  };
                });
              });
            }
          }
        })
      );

      return () => {
        cleanup.forEach((fn) => fn());

        // Remove rigid body
        const rigidBody = this.rigidBodies.get(node);
        if (rigidBody) this.#removeRigidBody(rigidBody);
        this.rigidBodies.delete(node);
      };
    });

    node.addEventListener("dispose", () => {
      const rigidBody = this.rigidBodies.get(node);
      if (rigidBody) this.#removeRigidBody(rigidBody);

      this.previousChildren.delete(node);
      this.primitiveColliders.delete(node);
      this.rigidBodies.delete(node);
    });
  }

  update(id: string, json: Partial<NodeJSON>) {
    const node = this.#scene.node.store.get(id);
    if (!node) throw new Error("Node not found");

    this.#scene.node.applyJSON(node, json);
  }

  remove(id: string) {
    const node = this.#scene.node.store.get(id);
    if (!node) throw new Error("Node not found");
    node.dispose();
  }

  #removeRigidBody(rigidBody: RigidBody) {
    // Remove colliders
    for (let i = 0; i < rigidBody.numColliders(); i++) {
      const collider = rigidBody.collider(i);
      if (collider) this.#scene.world.removeCollider(collider, false);
    }

    // Remove rigid body
    this.#scene.world.removeRigidBody(rigidBody);
  }

  #updateWorldTransform(node: Node) {
    // Update children
    node.listChildren().forEach((child) => this.#updateWorldTransform(child));

    // Update rigid body
    const rigidBody = this.rigidBodies.get(node);
    if (!rigidBody) return;

    const worldTranslation = node.getWorldTranslation();

    rigidBody.setNextKinematicTranslation({
      x: worldTranslation[0],
      y: worldTranslation[1],
      z: worldTranslation[2],
    });

    this.#updatePrimitiveColliders(node);
  }

  #updatePrimitiveColliders(node: Node) {
    const rigidBody = this.rigidBodies.get(node);
    if (!rigidBody) return;

    // Update trimesh colliders
    const colliderExtension = node.getExtension<ColliderExt>(ColliderExt.EXTENSION_NAME);
    if (!colliderExtension) return;

    const type = colliderExtension.getType();
    if (type !== "trimesh") return;

    const mesh = colliderExtension.getMesh();
    if (!mesh) return;

    const vertices = mesh.listPrimitives().map((primitive) => {
      const attribute = primitive.getAttribute("POSITION");
      const array = attribute?.getArray();
      return Float32Array.from(array ?? []);
    });

    const primitiveColliders = this.primitiveColliders.get(node);
    if (!primitiveColliders) return;

    const newPrimitiveColliders: Collider[] = [];

    primitiveColliders.forEach((collider, i) => {
      if (!collider || !(collider.shape instanceof TriMesh)) return;

      const rawVertices = vertices[i];
      if (!rawVertices) return;

      const newVertices = Float32Array.from(rawVertices);
      const newIndices = Uint32Array.from(collider.shape.indices);

      const worldMatrix = node.getWorldMatrix();

      // Apply transform to vertices
      for (let j = 0; j < newVertices.length; j += 3) {
        const x = newVertices[j] ?? 0;
        const y = newVertices[j + 1] ?? 0;
        const z = newVertices[j + 2] ?? 0;

        // Apply transform
        const e = worldMatrix;
        const w = 1 / (e[3] * x + e[7] * y + e[11] * z + e[15]);
        const newX = (e[0] * x + e[4] * y + e[8] * z + e[12]) * w;
        const newY = (e[1] * x + e[5] * y + e[9] * z + e[13]) * w;
        const newZ = (e[2] * x + e[6] * y + e[10] * z + e[14]) * w;

        // Subtract translation (we apply it on the rigid body)
        newVertices[j] = newX - e[12];
        newVertices[j + 1] = newY - e[13];
        newVertices[j + 2] = newZ - e[14];
      }

      // Remove old collider
      this.#scene.world.removeCollider(collider, true);

      // Create new collider
      const newColliderDesc = ColliderDesc.trimesh(newVertices, newIndices);
      newColliderDesc.setCollisionGroups(COLLISION_GROUP.static);

      newPrimitiveColliders.push(this.#scene.world.createCollider(newColliderDesc, rigidBody));
    });

    this.primitiveColliders.set(node, newPrimitiveColliders);
  }
}

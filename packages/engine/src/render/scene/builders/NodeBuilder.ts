import { Primitive } from "@gltf-transform/core";
import { Bone, Mesh as ThreeMesh, Object3D, Skeleton, SkinnedMesh } from "three";

import { NodeJSON } from "../../../scene";
import { subscribe } from "../../../utils/subscribe";
import { RenderScene } from "../RenderScene";
import { Builder } from "./Builder";

/**
 * @internal
 * Handles the conversion of nodes to Three.js objects.
 */
export class NodeBuilder extends Builder<NodeJSON, Bone | Object3D> {
  constructor(scene: RenderScene) {
    super(scene);
  }

  add(json: Partial<NodeJSON>, id: string) {
    const previousObject = this.getObject(id);
    if (previousObject) throw new Error(`Node with id ${id} already exists.`);

    const { object: node } = this.scene.node.create(json, id);

    // Isolate object creation from subscription logic
    // Objects may be created and disposed of multiple times
    {
      const object = new Object3D();
      this.scene.root.add(object);
      this.setObject(id, object);
    }

    this.subscribeToObject(id, (object) => {
      if (!object) return;

      const cleanup: Array<() => void> = [];

      cleanup.push(
        subscribe(node, "Name", (value) => {
          object.name = value;
        })
      );

      cleanup.push(
        subscribe(node, "Translation", (value) => {
          object.position.fromArray(value);
        })
      );

      cleanup.push(
        subscribe(node, "Rotation", (value) => {
          object.quaternion.fromArray(value);
        })
      );

      cleanup.push(
        subscribe(node, "Scale", (value) => {
          object.scale.fromArray(value);
        })
      );

      cleanup.push(
        subscribe(node, "Children", (children) => {
          // Add children
          children.forEach((child) => {
            const childId = this.scene.node.getId(child);
            if (!childId) throw new Error("Child id not found.");

            const childObject = this.getObject(childId);
            if (childObject) object.add(childObject);
          });

          return () => {
            // Remove children
            children.forEach((child) => {
              const childId = this.scene.node.getId(child);
              if (!childId) throw new Error("Child id not found.");

              const childObject = this.getObject(childId);
              if (childObject) object.remove(childObject);
            });
          };
        })
      );

      cleanup.push(
        subscribe(node, "Mesh", (mesh) => {
          if (!mesh) return;

          const meshCleanup: Array<() => void> = [];

          // Add mesh to object
          const meshId = this.scene.mesh.getId(mesh);
          if (!meshId) throw new Error("Mesh id not found.");

          const meshObject = this.scene.builders.mesh.getObject(meshId);
          if (!meshObject) throw new Error("Mesh object not found.");

          object.add(meshObject);

          // Apply weights
          meshCleanup.push(
            subscribe(node, "Weights", (weights) => {
              weights.forEach((weight, i) => {
                meshObject.traverse((child) => {
                  if (child instanceof ThreeMesh) {
                    if ("morphTargetInfluences" in child) {
                      if (child.morphTargetInfluences) child.morphTargetInfluences[i] = weight;
                    }
                  }
                });
              });
            })
          );

          // Bind mesh to skeleton
          meshCleanup.push(
            subscribe(node, "Skin", (skin) => {
              if (!skin) return;

              const skinId = this.scene.skin.getId(skin);
              if (!skinId) throw new Error("Skin id not found.");

              return this.scene.builders.skin.subscribeToObject(skinId, (skeleton) => {
                if (!skeleton) return;

                return subscribe(mesh, "Primitives", (primitives) => {
                  const primitivesCleanup: Array<() => void> = [];

                  primitives.forEach((primitive) => {
                    const primitiveId = this.scene.primitive.getId(primitive);
                    if (!primitiveId) throw new Error("Primitive id not found.");

                    // Convert mesh to skinned mesh
                    this.#primitiveToSkinnedMesh(primitive);

                    // Bind mesh to skeleton
                    primitivesCleanup.push(
                      this.scene.builders.primitive.subscribeToObject(
                        primitiveId,
                        (primitiveObject) => {
                          if (!(primitiveObject instanceof SkinnedMesh)) return;

                          let bindMatrix = primitiveObject.matrix;

                          const skeletonRoot = skin.getSkeleton();
                          if (skeletonRoot) {
                            const skeletonRootId = this.scene.node.getId(skeletonRoot);
                            if (skeletonRootId) {
                              const skeletonRootObject = this.getObject(skeletonRootId);
                              if (skeletonRootObject) {
                                bindMatrix = skeletonRootObject.matrix;
                              }
                            }
                          }

                          primitiveObject.bind(skeleton, bindMatrix);
                        }
                      )
                    );
                  });

                  return () => {
                    // Convert primitves back to normal meshes
                    primitives.forEach((primitive) => {
                      this.#primitiveToMesh(primitive);
                    });

                    primitivesCleanup.forEach((fn) => fn());
                  };
                });
              });
            })
          );

          return () => {
            object.remove(meshObject);

            meshCleanup.forEach((fn) => fn());
          };
        })
      );

      return () => {
        cleanup.forEach((fn) => fn());
      };
    });

    node.addEventListener("dispose", () => {
      // Restore primitive objects
      const mesh = node.getMesh();
      if (mesh) {
        mesh.listPrimitives().forEach((primitive) => {
          this.#primitiveToMesh(primitive);
        });
      }

      const object = this.getObject(id);
      if (object) object.removeFromParent();

      this.setObject(id, null);
    });

    return node;
  }

  remove(id: string) {
    this.scene.node.store.get(id)?.dispose();
  }

  update(json: Partial<NodeJSON>, id: string) {
    const node = this.scene.node.store.get(id);
    if (!node) throw new Error(`Node with id ${id} does not exist.`);

    this.scene.node.applyJSON(node, json);
  }

  #primitiveToSkinnedMesh(primitive: Primitive) {
    const primitiveId = this.scene.primitive.getId(primitive);
    if (!primitiveId) throw new Error("Primitive id not found.");

    const primitiveObject = this.scene.builders.primitive.getObject(primitiveId);
    if (!primitiveObject) throw new Error("Primitive object not found.");

    // Convert primitive to skinned mesh
    const newPrimitiveObject = new SkinnedMesh();
    newPrimitiveObject.bind(new Skeleton([]));
    this.scene.builders.primitive.setObject(primitiveId, newPrimitiveObject);

    newPrimitiveObject.castShadow = primitiveObject.castShadow;
    newPrimitiveObject.receiveShadow = primitiveObject.receiveShadow;
    newPrimitiveObject.geometry = primitiveObject.geometry;
    newPrimitiveObject.material = primitiveObject.material;
    if (primitiveObject.morphTargetInfluences)
      newPrimitiveObject.morphTargetInfluences = [...primitiveObject.morphTargetInfluences];
    if (primitiveObject.morphTargetDictionary)
      newPrimitiveObject.morphTargetDictionary = {
        ...primitiveObject.morphTargetDictionary,
      };

    // Normalize skin weights
    if (!newPrimitiveObject.geometry.attributes.skinWeight?.normalized)
      newPrimitiveObject.normalizeSkinWeights();

    // Remove old primitive object
    primitiveObject.removeFromParent();

    return newPrimitiveObject;
  }

  #primitiveToMesh(primitive: Primitive) {
    const primitiveId = this.scene.primitive.getId(primitive);
    if (!primitiveId) throw new Error("Primitive id not found.");

    const primitiveObject = this.scene.builders.primitive.getObject(primitiveId);
    if (!primitiveObject) throw new Error("Primitive object not found.");

    // Convert primitive to skinned mesh
    const newPrimitiveObject = new ThreeMesh();
    this.scene.builders.primitive.setObject(primitiveId, newPrimitiveObject);

    newPrimitiveObject.castShadow = primitiveObject.castShadow;
    newPrimitiveObject.receiveShadow = primitiveObject.receiveShadow;
    newPrimitiveObject.geometry = primitiveObject.geometry;
    newPrimitiveObject.material = primitiveObject.material;
    if (primitiveObject.morphTargetInfluences)
      newPrimitiveObject.morphTargetInfluences = [...primitiveObject.morphTargetInfluences];
    if (primitiveObject.morphTargetDictionary)
      newPrimitiveObject.morphTargetDictionary = {
        ...primitiveObject.morphTargetDictionary,
      };

    // Remove old primitive object
    primitiveObject.removeFromParent();

    return newPrimitiveObject;
  }
}

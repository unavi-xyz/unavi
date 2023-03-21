import { Primitive } from "@gltf-transform/core";
import { VRM, VRMLoaderPlugin, VRMUtils } from "@pixiv/three-vrm";
import { Avatar } from "@wired-labs/gltf-extensions";
import {
  Bone,
  BufferGeometry,
  Mesh,
  Mesh as ThreeMesh,
  Object3D,
  Skeleton,
  SkinnedMesh,
} from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";
import { MeshBVHVisualizer, StaticGeometryGenerator } from "three-mesh-bvh";

import { DEFAULT_VISUALS } from "../../../constants";
import { NodeJSON } from "../../../scene";
import { subscribe } from "../../../utils/subscribe";
import { deepDispose } from "../../utils/deepDispose";
import { Builder } from "./Builder";

/**
 * @internal
 * Handles the conversion of nodes to Three.js objects.
 */
export class NodeBuilder extends Builder<NodeJSON, Bone | Object3D> {
  avatarObjects = new Map<string, Object3D>();

  bvhHelpers = new Map<StaticGeometryGenerator, MeshBVHVisualizer>();
  meshHelpers = new Map<StaticGeometryGenerator, Mesh>();
  bvhGenerators = new Map<string, StaticGeometryGenerator>();

  #visuals = DEFAULT_VISUALS;

  #newMeshHelper(generator: StaticGeometryGenerator) {
    const meshHelper = new Mesh(new BufferGeometry());
    meshHelper.visible = false;
    this.scene.root.add(meshHelper);
    this.meshHelpers.set(generator, meshHelper);

    const bvhHelper = new MeshBVHVisualizer(meshHelper, 10);
    bvhHelper.visible = this.#visuals;
    bvhHelper.frustumCulled = false;
    this.scene.root.add(bvhHelper);
    this.bvhHelpers.set(generator, bvhHelper);

    return meshHelper;
  }

  regenerateMeshBVH() {
    this.bvhGenerators.forEach((generator) => {
      const meshHelper = this.meshHelpers.get(generator) ?? this.#newMeshHelper(generator);

      const bvhHelper = this.bvhHelpers.get(generator);
      if (!bvhHelper) throw new Error("BVH helper not found.");

      generator.generate(meshHelper.geometry);

      if (meshHelper.geometry.boundsTree) meshHelper.geometry.boundsTree.refit();
      else meshHelper.geometry.computeBoundsTree();

      bvhHelper.update();
    });
  }

  setBvhVisuals(visible: boolean) {
    this.#visuals = visible;
    this.bvhHelpers.forEach((bvhHelper) => {
      bvhHelper.visible = visible;
    });
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
              if (childObject) {
                object.remove(childObject);
                this.scene.root.add(childObject);
              }
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

          const meshObject = this.scene.builders.mesh.getUniqueObject(meshId, id);
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
            this.scene.builders.mesh.removeObject(meshObject);
            meshCleanup.forEach((fn) => fn());
          };
        })
      );

      let vrmUri = "";

      cleanup.push(
        subscribe(node, "Extensions", (extensions) => {
          const avatarObject = this.avatarObjects.get(id);

          const avatar = extensions.find((ext): ext is Avatar => ext instanceof Avatar);
          if (!avatar) {
            vrmUri = "";
            if (avatarObject) {
              object.remove(avatarObject);
              deepDispose(avatarObject);
              this.avatarObjects.delete(id);
            }
            return;
          }

          const uri = avatar.getURI();
          if (!uri || uri === vrmUri) return;

          if (avatarObject) {
            object.remove(avatarObject);
            deepDispose(avatarObject);
            this.avatarObjects.delete(id);
          }

          vrmUri = uri;

          const loader = new GLTFLoader();
          loader.setCrossOrigin("anonymous");
          loader.register((parser) => new VRMLoaderPlugin(parser));

          loader.load(uri, (gltf) => {
            const vrm = gltf.userData.vrm as VRM;
            vrm.scene.rotateY(Math.PI);

            VRMUtils.removeUnnecessaryVertices(vrm.scene);
            VRMUtils.removeUnnecessaryJoints(vrm.scene);
            VRMUtils.rotateVRM0(vrm);

            vrm.scene.traverse((obj) => {
              if (obj instanceof Mesh) {
                obj.castShadow = true;
                obj.geometry.computeBoundsTree();
              }
            });

            // Generate BVH
            const generator = new StaticGeometryGenerator(vrm.scene);
            this.bvhGenerators.set(id, generator);

            // Add VRM to object
            object.add(vrm.scene);
            this.avatarObjects.set(id, vrm.scene);
          });

          return () => {
            const generator = this.bvhGenerators.get(id);
            if (generator) {
              this.bvhGenerators.delete(id);
              const helper = this.meshHelpers.get(generator);
              const bvhHelper = this.bvhHelpers.get(generator);

              if (helper) {
                this.meshHelpers.delete(generator);
                helper.removeFromParent();
                helper.geometry.dispose();
              }

              if (bvhHelper) {
                this.bvhHelpers.delete(generator);
                bvhHelper.removeFromParent();
                bvhHelper.traverse((obj) => {
                  if (obj instanceof Mesh) {
                    obj.geometry.dispose();
                  }
                });
              }
            }
          };
        })
      );

      return () => {
        cleanup.forEach((fn) => fn());

        const avatarObject = this.avatarObjects.get(id);
        if (avatarObject) {
          object.remove(avatarObject);
          deepDispose(avatarObject);
          this.avatarObjects.delete(id);
        }
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
    if (
      newPrimitiveObject.geometry.attributes.skinWeight &&
      !newPrimitiveObject.geometry.attributes.skinWeight.normalized
    )
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

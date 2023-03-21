import { Accessor } from "@gltf-transform/core";
import { BufferAttribute, Mesh, MeshStandardMaterial, SkinnedMesh } from "three";

import { PrimitiveJSON } from "../../../scene";
import { subscribe } from "../../../utils/subscribe";
import { THREE_ATTRIBUTE_NAMES } from "../constants";
import { Builder } from "./Builder";

export const DEFAULT_MATERIAL = new MeshStandardMaterial();

/**
 * @internal
 * Handles the conversion of primitives to Three.js objects.
 */
export class PrimitiveBuilder extends Builder<PrimitiveJSON, Mesh | SkinnedMesh> {
  objectClones = new Map<string, Mesh[]>();
  #setObjectTimeouts = new Map<string, NodeJS.Timeout>();

  override setObject(id: string, object: Mesh | SkinnedMesh | null) {
    super.setObject(id, object);

    if (object) {
      // Clear any existing timeout
      const prevTimeout = this.#setObjectTimeouts.get(id);
      if (prevTimeout) clearTimeout(prevTimeout);

      // Disable frustum culling initially, to force loading of all textures
      object.frustumCulled = false;

      // Enable frustum culling after a delay
      const timeout = setTimeout(() => {
        object.frustumCulled = true;
        this.#setObjectTimeouts.delete(id);
      }, 10000);

      this.#setObjectTimeouts.set(id, timeout);
    }
  }

  getUniqueObject(id: string): Mesh | SkinnedMesh | undefined {
    const baseObject = this.getObject(id);
    if (!baseObject) return undefined;

    // If base object is not used yet, use it
    if (!baseObject.parent) return baseObject;

    // Any additional uses of the base object will be clones
    const clone = baseObject.clone();
    const clones = this.objectClones.get(id) ?? [];
    clones.push(clone);
    this.objectClones.set(id, clones);

    return clone;
  }

  removeObject(object: Mesh | SkinnedMesh) {
    // Remove from scene
    object.removeFromParent();

    // Remove from list of clones
    for (const [id, clones] of this.objectClones) {
      const index = clones.indexOf(object);
      if (index === -1) continue;

      clones.splice(index, 1);
      if (clones.length === 0) this.objectClones.delete(id);
      break;
    }
  }

  add(json: Partial<PrimitiveJSON>, id: string) {
    const previousObject = this.getObject(id);
    if (previousObject) throw new Error(`Primitive with id ${id} already exists.`);

    const { object: primitive } = this.scene.primitive.create(json, id);

    // Isolate object creation from subscription logic
    // Objects may be created and disposed of multiple times
    {
      const object = new Mesh();
      object.geometry.morphTargetsRelative = true;
      object.castShadow = true;
      object.receiveShadow = true;

      this.scene.root.add(object);
      this.setObject(id, object);
    }

    this.subscribeToObject(id, (object) => {
      if (!object) return;

      const cleanup: Array<() => void> = [];

      cleanup.push(
        subscribe(primitive, "Name", (value) => {
          object.name = value;
        })
      );

      cleanup.push(
        subscribe(primitive, "Material", (value) => {
          if (value) {
            const materialId = this.scene.material.getId(value);
            if (!materialId) throw new Error("Material id not found.");

            const materialObject = this.scene.builders.material.getObject(materialId);
            if (!materialObject) throw new Error("Material object not found.");

            return subscribe(primitive, "Semantics", (names) => {
              const useDerivativeTangents = !names.includes("TANGENT");
              const useVertexColors = names.includes("COLOR_0");
              const useFlatShading = !names.includes("NORMAL");

              if (useDerivativeTangents || useVertexColors || useFlatShading) {
                // Clone material to avoid modifying the original
                const clonedMaterial = materialObject.clone();

                // Setup CSM
                this.scene.csm?.setupMaterial(clonedMaterial);

                if (useDerivativeTangents) clonedMaterial.normalScale.y *= -1;
                if (useVertexColors) clonedMaterial.vertexColors = true;
                if (useFlatShading) clonedMaterial.flatShading = true;

                object.material = clonedMaterial;

                return () => {
                  // Dispose of cloned material
                  clonedMaterial.dispose();
                };
              } else {
                object.material = materialObject;
              }
            });
          } else {
            object.material = DEFAULT_MATERIAL;
          }
        })
      );

      cleanup.push(
        subscribe(primitive, "Indices", (value) => {
          if (value) {
            const attribute = accessorToAttribute(value);
            object.geometry.setIndex(attribute);
          } else {
            object.geometry.setIndex(null);
          }
        })
      );

      cleanup.push(
        subscribe(primitive, "Attributes", (accessors) => {
          return subscribe(primitive, "Semantics", (names) => {
            // Add new attributes
            accessors.forEach((accessor, i) => {
              const name = names[i];
              if (name === undefined) throw new Error("Semantics not found");

              const threeName = THREE_ATTRIBUTE_NAMES[name as keyof typeof THREE_ATTRIBUTE_NAMES];

              const attribute = accessorToAttribute(accessor);

              if (attribute) object.geometry.setAttribute(threeName, attribute);
              else object.geometry.deleteAttribute(threeName);
            });

            // Calculate BVH
            object.geometry.computeBoundsTree();

            return () => {
              // Remove all attributes
              Object.values(THREE_ATTRIBUTE_NAMES).forEach((name) => {
                object.geometry.deleteAttribute(name);
              });

              // Calculate BVH
              object.geometry.computeBoundsTree();
            };
          });
        })
      );

      cleanup.push(
        subscribe(primitive, "Targets", (value) => {
          // Reset morph influences
          if (value.length === 0) object.morphTargetInfluences = undefined;
          else object.morphTargetInfluences = [];

          // Add new morph attributes
          value.forEach((target) => {
            const names = target.listSemantics();

            target.listAttributes().forEach((accessor, i) => {
              const name = names[i];
              if (name === undefined) throw new Error("Semantics not found");

              const threeName = THREE_ATTRIBUTE_NAMES[name as keyof typeof THREE_ATTRIBUTE_NAMES];

              const attribute = accessorToAttribute(accessor);

              if (attribute) {
                const morphAttributes = object.geometry.morphAttributes[threeName];
                if (morphAttributes) morphAttributes.push(attribute);
                else object.geometry.morphAttributes[threeName] = [attribute];
              } else {
                delete object.geometry.morphAttributes[threeName];
              }
            });
          });

          return () => {
            // Remove morph attributes
            Object.values(THREE_ATTRIBUTE_NAMES).forEach((name) => {
              delete object.geometry.morphAttributes[name];
            });
          };
        })
      );

      return () => {
        cleanup.forEach((fn) => fn());
      };
    });

    primitive.addEventListener("dispose", () => {
      const object = this.getObject(id);
      if (object) {
        object.removeFromParent();
        object.geometry.dispose();
      }

      this.setObject(id, null);
    });

    return primitive;
  }

  remove(id: string) {
    this.scene.primitive.store.get(id)?.dispose();
  }

  update(json: Partial<PrimitiveJSON>, id: string) {
    const primitive = this.scene.primitive.store.get(id);
    if (!primitive) throw new Error(`Primitive with id ${id} does not exist.`);

    this.scene.primitive.applyJSON(primitive, json);
  }
}

function accessorToAttribute(accessor: Accessor) {
  const array = accessor.getArray();
  if (!array) return null;

  return new BufferAttribute(array, accessor.getElementSize(), accessor.getNormalized());
}

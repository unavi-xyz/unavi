import { Mesh } from "@gltf-transform/core";
import { Event, Mesh as ThreeMesh, Object3D } from "three";

import { MeshJSON } from "../../../scene";
import { subscribe } from "../../../utils/subscribe";
import { Builder } from "./Builder";

/**
 * @internal
 * Handles the conversion of meshes to Three.js objects.
 */
export class MeshBuilder extends Builder<Mesh, MeshJSON, Object3D> {
  objectClones = new Map<string, Object3D[]>();
  primitiveClones = new Map<string, Object3D[]>();

  #uniqueObjectCleanup = new Map<Object3D, () => void>();

  getUniqueObject(id: string, nodeId: string): Object3D<Event> | undefined {
    const baseObject = this.getObject(id);
    if (!baseObject) return undefined;

    // If base object is not used yet, use it
    if (!baseObject.parent) return baseObject;

    // Any additional uses of the base object will be clones
    const clone = new Object3D();
    clone.name = baseObject.name;

    const mesh = this.scene.mesh.store.get(id);
    if (!mesh) throw new Error("Mesh not found.");

    const unsubscribe = subscribe(mesh, "Primitives", (value) => {
      const cleanup: Array<() => void> = value.map((primitive) => {
        const primitiveId = this.scene.primitive.getId(primitive);
        if (!primitiveId) throw new Error("Primitive id not found.");

        return this.scene.builders.primitive.subscribeToObject(primitiveId, (primitiveObject) => {
          if (!primitiveObject) return;

          const primitiveId = this.scene.primitive.getId(primitive);
          if (!primitiveId) throw new Error("Primitive id not found.");

          const clonedPrimitive = this.scene.builders.primitive.getUniqueObject(primitiveId);
          if (!clonedPrimitive) throw new Error("Cloned primitive not found.");

          clone.add(clonedPrimitive);

          const primitiveClones = this.primitiveClones.get(nodeId) ?? [];
          primitiveClones.push(clonedPrimitive);
          this.primitiveClones.set(nodeId, primitiveClones);

          return () => {
            this.scene.builders.primitive.removeObject(clonedPrimitive);

            const primitiveClones = this.primitiveClones.get(nodeId) ?? [];
            const index = primitiveClones.indexOf(clonedPrimitive);
            if (index === -1) return;

            primitiveClones.splice(index, 1);
            if (primitiveClones.length === 0) this.primitiveClones.delete(nodeId);

            this.primitiveClones.set(nodeId, primitiveClones);
          };
        });
      });

      return () => {
        cleanup.forEach((fn) => fn());
      };
    });

    const clones = this.objectClones.get(id) ?? [];
    clones.push(clone);
    this.objectClones.set(id, clones);

    this.#uniqueObjectCleanup.set(clone, unsubscribe);

    return clone;
  }

  removeObject(object: Object3D) {
    // Cleanup
    const cleanup = this.#uniqueObjectCleanup.get(object);
    if (cleanup) cleanup();
    this.#uniqueObjectCleanup.delete(object);

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

  add(json: Partial<MeshJSON>, id: string) {
    const prevObject = this.scene.mesh.store.get(id);
    if (prevObject) throw new Error(`Mesh with id ${id} already exists.`);

    const { object: mesh } = this.scene.mesh.create(json, id);

    const baseObject = new Object3D();
    this.setObject(id, baseObject);

    subscribe(mesh, "Name", (value) => {
      this.#updateObject(id, (obj) => {
        obj.name = value;
      });
    });

    subscribe(mesh, "Primitives", (value) => {
      const cleanup: Array<() => void> = [];

      // Add new primitives
      value.forEach((primitive) => {
        const primitiveId = this.scene.primitive.getId(primitive);
        if (!primitiveId) throw new Error("Primitive id not found.");

        cleanup.push(
          this.scene.builders.primitive.subscribeToObject(primitiveId, (primitiveObject) => {
            if (!primitiveObject) return;

            baseObject.add(primitiveObject);

            return () => {
              baseObject.remove(primitiveObject);
            };
          })
        );
      });

      // Apply weights
      cleanup.push(
        subscribe(mesh, "Weights", (value) => {
          value.forEach((weight, i) => {
            this.#updateObject(id, (obj) => {
              obj.traverse((child) => {
                if (child instanceof ThreeMesh) {
                  if ("morphTargetInfluences" in child) {
                    if (child.morphTargetInfluences) child.morphTargetInfluences[i] = weight;
                  }
                }
              });
            });
          });
        })
      );

      return () => {
        cleanup.forEach((fn) => fn());
      };
    });

    mesh.addEventListener("dispose", () => {
      this.#updateObject(id, (obj) => {
        obj.removeFromParent();
      });

      this.setObject(id, null);
    });

    return mesh;
  }

  remove(id: string) {
    const clones = this.objectClones.get(id);
    if (clones) {
      clones.forEach((clone) => clone.removeFromParent());
      this.objectClones.delete(id);
    }

    this.scene.mesh.store.get(id)?.dispose();
  }

  update(json: Partial<MeshJSON>, id: string) {
    const mesh = this.scene.mesh.store.get(id);
    if (!mesh) throw new Error(`Mesh with id ${id} does not exist.`);

    this.scene.mesh.applyJSON(mesh, json);
  }

  /**
   * Runs a function on the base object and all clones.
   * @param id The id of the object.
   * @param fn The function to run.
   */
  #updateObject(id: string, fn: (object: Object3D) => void) {
    const baseObject = this.getObject(id);
    if (!baseObject) throw new Error(`Object with id ${id} does not exist.`);

    const clones = this.objectClones.get(id) ?? [];
    const allObjects = [baseObject, ...clones];

    allObjects.forEach(fn);
  }
}

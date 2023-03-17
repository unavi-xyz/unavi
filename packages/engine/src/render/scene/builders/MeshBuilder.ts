import { Mesh, Object3D } from "three";

import { MeshJSON } from "../../../scene";
import { subscribe } from "../../../utils/subscribe";
import { RenderScene } from "../RenderScene";
import { Builder } from "./Builder";

/**
 * @internal
 * Handles the conversion of meshes to Three.js objects.
 */
export class MeshBuilder extends Builder<MeshJSON, Object3D> {
  constructor(scene: RenderScene) {
    super(scene);
  }

  add(json: Partial<MeshJSON>, id: string) {
    const previousObject = this.getObject(id);
    if (previousObject) throw new Error(`Mesh with id ${id} already exists.`);

    const { object: mesh } = this.scene.mesh.create(json, id);

    const object = new Object3D();
    this.scene.root.add(object);
    this.setObject(id, object);

    subscribe(mesh, "Name", (value) => {
      object.name = value;
    });

    subscribe(mesh, "Primitives", (value) => {
      const cleanup: Array<() => void> = [];

      // Add new primitives
      value.forEach((primitive) => {
        const primitiveId = this.scene.primitive.getId(primitive);
        if (!primitiveId) throw new Error("Primitive id not found.");

        cleanup.push(
          this.scene.builders.primitive.subscribeToObject(primitiveId, (primitiveObject) => {
            if (primitiveObject) object.add(primitiveObject);
          })
        );
      });

      // Apply weights
      cleanup.push(
        subscribe(mesh, "Weights", (value) => {
          value.forEach((weight, i) => {
            object.traverse((child) => {
              if (child instanceof Mesh) {
                if ("morphTargetInfluences" in child) {
                  if (child.morphTargetInfluences) child.morphTargetInfluences[i] = weight;
                }
              }
            });
          });
        })
      );

      return () => {
        // Remove primitives
        value.forEach((primitive) => {
          const primitiveId = this.scene.primitive.getId(primitive);
          if (!primitiveId) throw new Error("Primitive id not found.");

          const primitiveObject = this.scene.builders.primitive.getObject(primitiveId);
          if (primitiveObject) object.remove(primitiveObject);
        });

        cleanup.forEach((fn) => fn());
      };
    });

    mesh.addEventListener("dispose", () => {
      object.removeFromParent();
      this.setObject(id, null);
    });

    return mesh;
  }

  remove(id: string) {
    this.scene.mesh.store.get(id)?.dispose();
  }

  update(json: Partial<MeshJSON>, id: string) {
    const mesh = this.scene.mesh.store.get(id);
    if (!mesh) throw new Error(`Mesh with id ${id} does not exist.`);

    this.scene.mesh.applyJSON(mesh, json);
  }
}

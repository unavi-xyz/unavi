import { Node } from "@gltf-transform/core";
import { Bone, Matrix4, Object3D, Skeleton } from "three";

import { SkinJSON } from "../../../scene";
import { subscribe } from "../../../utils/subscribe";
import { RenderScene } from "../RenderScene";
import { Builder } from "./Builder";

/**
 * @internal
 * Handles the conversion of skins to Three.js objects.
 */
export class SkinBuilder extends Builder<SkinJSON, Skeleton> {
  constructor(scene: RenderScene) {
    super(scene);
  }

  add(json: Partial<SkinJSON>, id: string) {
    const { object: skin } = this.scene.skin.create(json, id);

    subscribe(skin, "Joints", (joints) => {
      const cleanup: Array<() => void> = [];

      // Convert joint objects to bones
      const bones = joints.map((node) => {
        return this.#nodeToBone(node);
      });

      // Create skeleton
      const skeleton = new Skeleton(bones);
      this.setObject(id, skeleton);

      // Apply inverse bind matrices
      cleanup.push(
        subscribe(skin, "InverseBindMatrices", (matrices) => {
          const array = matrices?.getArray();
          if (!array) return;

          const boneInverses = bones.map((bone, i) => {
            return new Matrix4().fromArray(array, i * 16);
          });

          skeleton.boneInverses = boneInverses;

          return () => {
            skeleton.boneInverses = [];
          };
        })
      );

      return () => {
        // Convert joint objects back to nodes
        joints.forEach((node) => {
          this.#nodeToObject(node);
        });

        cleanup.forEach((fn) => fn());
      };
    });

    skin.addEventListener("dispose", () => {
      // Convert joint objects back to nodes
      skin.listJoints().forEach((node) => {
        this.#nodeToObject(node);
      });
    });

    return skin;
  }

  remove(id: string) {
    this.scene.skin.store.get(id)?.dispose();
  }

  update(json: Partial<SkinJSON>, id: string) {
    const skin = this.scene.skin.store.get(id);
    if (!skin) throw new Error(`Skin with id ${id} does not exist.`);

    this.scene.skin.applyJSON(skin, json);
  }

  #nodeToObject(node: Node) {
    const nodeId = this.scene.node.getId(node);
    if (!nodeId) throw new Error("Joint id not found.");

    const nodeObject = this.scene.builders.node.getObject(nodeId);
    if (!nodeObject) throw new Error("Joint object not found.");

    const newObject = new Object3D();

    newObject.parent = nodeObject.parent;
    newObject.children = [...nodeObject.children];
    newObject.matrix.copy(nodeObject.matrix);

    nodeObject.removeFromParent();

    this.scene.builders.node.setObject(nodeId, newObject);

    return newObject;
  }

  #nodeToBone(node: Node) {
    const nodeId = this.scene.node.getId(node);
    if (!nodeId) throw new Error("Joint id not found.");

    const nodeObject = this.scene.builders.node.getObject(nodeId);
    if (!nodeObject) throw new Error("Joint object not found.");

    const newObject = new Bone();

    newObject.parent = nodeObject.parent;
    newObject.children = [...nodeObject.children];
    newObject.matrix.copy(nodeObject.matrix);

    nodeObject.removeFromParent();

    this.scene.builders.node.setObject(nodeId, newObject);

    return newObject;
  }
}

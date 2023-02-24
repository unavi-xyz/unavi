import { IScene } from "@wired-labs/behave-graph-core";

import { Engine } from "../Engine";
import { ValueType } from "../gltf/extensions/Behavior/types";
import { parseJSONPath } from "./parseJsonPath";

/**
 * Used by behavior nodes to access the scene.
 */
export class BehaviorScene implements IScene {
  #engine: Engine;

  constructor(engine: Engine) {
    this.#engine = engine;
  }

  getProperty(jsonPath: string, valueType: string) {
    const parsed = parseJSONPath(jsonPath);
    if (!parsed) return;

    const { resource, index, property } = parsed;

    switch (resource) {
      case "nodes": {
        const node = this.#engine.scene.doc.getRoot().listNodes()[index];
        if (!node) throw new Error("Invalid node index");

        switch (property) {
          case "translation": {
            if (valueType !== ValueType.vec3) return;

            const translation = node.getTranslation();

            return {
              x: translation[0],
              y: translation[1],
              z: translation[2],
            };
          }

          case "rotation": {
            if (valueType !== ValueType.quat && valueType !== ValueType.vec4) return;

            const rotation = node.getRotation();

            return {
              x: rotation[0],
              y: rotation[1],
              z: rotation[2],
              w: rotation[3],
            };
          }

          case "scale": {
            if (valueType !== ValueType.vec3) return;

            const scale = node.getScale();

            return {
              x: scale[0],
              y: scale[1],
              z: scale[2],
            };
          }
        }
        break;
      }

      default: {
        throw new Error("Invalid resource");
      }
    }
  }

  setProperty(jsonPath: string, valueType: string, value: any): void {
    const parsed = parseJSONPath(jsonPath);
    if (!parsed) return;

    const { resource, index, property } = parsed;

    switch (resource) {
      case "nodes": {
        const node = this.#engine.scene.doc.getRoot().listNodes()[index];
        if (!node) throw new Error("Invalid node index");

        switch (property) {
          case "translation": {
            if (valueType !== ValueType.vec3) return;
            node.setTranslation([value.x, value.y, value.z]);
            break;
          }

          case "rotation": {
            if (valueType !== ValueType.vec4) return;
            node.setRotation([value.x, value.y, value.z, value.w]);
            break;
          }

          case "scale": {
            if (valueType !== ValueType.vec3) return;
            node.setScale([value.x, value.y, value.z]);
            break;
          }
        }
        break;
      }

      default: {
        throw new Error("Invalid resource");
      }
    }
  }

  addOnClickedListener(jsonPath: string, callback: (jsonPath: string) => void): void {}
  removeOnClickedListener(jsonPath: string, callback: (jsonPath: string) => void): void {}
}

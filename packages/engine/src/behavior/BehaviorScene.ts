import { IScene } from "@behave-graph/core";

import { Engine } from "../Engine";

export class BehaviorScene implements IScene {
  #engine: Engine;

  constructor(engine: Engine) {
    this.#engine = engine;
  }

  getProperty(jsonPath: string, valueTypeName: string) {
    const parsed = parseJSONPath(jsonPath);
    if (!parsed) return;

    const { resource, index, property } = parsed;

    switch (resource) {
      case "nodes": {
        const node = this.#engine.scene.doc.getRoot().listNodes()[index];
        if (!node) throw new Error("Invalid node index");

        switch (property) {
          case "name": {
            return node.getName();
          }

          case "translation": {
            return node.getTranslation();
          }

          case "rotation": {
            return node.getRotation();
          }

          case "scale": {
            return node.getScale();
          }
        }
        break;
      }

      default: {
        throw new Error("Invalid resource");
      }
    }
  }

  setProperty(jsonPath: string, valueTypeName: string, value: any): void {
    const parsed = parseJSONPath(jsonPath);
    if (!parsed) return;

    const { resource, index, property } = parsed;

    switch (resource) {
      case "nodes": {
        const node = this.#engine.scene.doc.getRoot().listNodes()[index];
        if (!node) throw new Error("Invalid node index");

        switch (property) {
          case "name": {
            node.setName(value);
            break;
          }

          case "translation": {
            node.setTranslation(value);
            break;
          }

          case "rotation": {
            node.setRotation(value);
            break;
          }

          case "scale": {
            node.setScale(value);
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
}

function parseJSONPath(jsonPath: string) {
  const parts = jsonPath.split("/") as [string, string, string];
  if (parts.length !== 3) return null;

  const resource = parts[0];
  const index = parseInt(parts[1]);
  const property = parts[2];

  return { resource, index, property };
}

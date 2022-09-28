import { Entity, SceneJSON, SceneMessage } from "../scene";
import { Scene } from "../scene/Scene";
import { PostMessage } from "../types";
import { FromGameMessage } from "./types";

/*
 * Wrapper around {@link Scene} for the game thread.
 * Syncs state with the {@link MainScene}.
 */
export class GameScene {
  #toMainThread: PostMessage<FromGameMessage>;

  #scene = new Scene();

  constructor(postMessage: PostMessage<FromGameMessage>) {
    this.#toMainThread = postMessage;
  }

  get entities$() {
    return this.#scene.entities$;
  }

  get materials$() {
    return this.#scene.materials$;
  }

  get entities() {
    return this.#scene.entities;
  }

  set entities(entities: { [id: string]: Entity }) {
    this.#scene.entities = entities;
  }

  onmessage = (event: MessageEvent<SceneMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "load_json":
        this.#scene.loadJSON(data.scene);
        break;
      case "add_entity":
        this.#scene.addEntity(Entity.fromJSON(data.entity));
        break;
      case "remove_entity":
        this.#scene.removeEntity(data.entityId);
        break;
      case "update_entity":
        this.#scene.updateEntity(data.entityId, data.data);
        break;
      case "update_global_transform":
        this.#scene.updateGlobalTransform(
          data.entityId,
          data.position,
          data.quaternion
        );
        break;
    }
  };

  toJSON(): SceneJSON {
    return this.#scene.toJSON();
  }

  destroy() {
    this.#scene.destroy();
  }
}

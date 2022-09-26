import {
  Entity,
  EntityJSON,
  Material,
  SceneJSON,
  SceneMessage,
} from "../scene";
import { Scene } from "../scene/Scene";
import { PostMessage, Quad, Triplet } from "../types";
import { FromRenderMessage } from "./types";

/*
 * Wrapper around {@link Scene} for the render thread.
 * Syncs state with the {@link MainScene}.
 */
export class RenderScene {
  #toMainThread: PostMessage<FromRenderMessage>;

  #scene = new Scene();

  constructor(postMessage: PostMessage<FromRenderMessage>) {
    this.#toMainThread = postMessage;
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
      case "add_material":
        this.#scene.addMaterial(Material.fromJSON(data.material));
        break;
      case "remove_material":
        this.#scene.removeMaterial(data.materialId);
        break;
      case "update_material":
        this.#scene.updateMaterial(data.materialId, data.data);
        break;
    }
  };

  get entities$() {
    return this.#scene.entities$;
  }

  get entities() {
    return this.#scene.entities;
  }

  get materials$() {
    return this.#scene.materials$;
  }

  get materials() {
    return this.#scene.materials;
  }

  get textures$() {
    return this.#scene.textures$;
  }

  get textures() {
    return this.#scene.textures;
  }

  get accessors() {
    return this.#scene.accessors;
  }

  get images$() {
    return this.#scene.images$;
  }

  get images() {
    return this.#scene.images;
  }

  updateEntity(entityId: string, data: Partial<EntityJSON>) {
    this.#scene.updateEntity(entityId, data);

    this.#toMainThread({
      subject: "update_entity",
      data: { entityId, data },
    });
  }

  updateGlobalTransform(entityId: string, position: Triplet, quaternion: Quad) {
    this.#scene.updateGlobalTransform(entityId, position, quaternion);

    this.#toMainThread({
      subject: "update_global_transform",
      data: {
        entityId,
        position,
        quaternion,
      },
    });
  }

  toJSON(): SceneJSON {
    return this.#scene.toJSON();
  }

  destroy() {
    this.#scene.destroy();
  }
}

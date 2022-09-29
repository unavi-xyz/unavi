import { GameThread } from "../game/GameThread";
import { ToGameMessage } from "../game/types";
import { LoaderThread } from "../loader/LoaderThread";
import { ToLoaderMessage } from "../loader/types";
import { RenderThread } from "../render/RenderThread";
import { ToRenderMessage } from "../render/types";
import { Entity } from "../scene/Entity";
import { Material } from "../scene/Material";
import { Scene } from "../scene/Scene";
import {
  EntityJSON,
  MaterialJSON,
  SceneJSON,
  SceneMessage,
} from "../scene/types";
import { PostMessage, Quad, Triplet } from "../types";

/*
 * Wrapper around {@link Scene} for the main thread.
 * Syncs state with the {@link RenderScene} and {@link GameScene}.
 */
export class MainScene {
  #toGameThread: PostMessage<ToGameMessage>;
  #toLoaderThread: PostMessage<ToLoaderMessage>;
  #toRenderThread: PostMessage<ToRenderMessage>;

  #scene = new Scene();

  constructor({
    gameThread,
    loaderThread,
    renderThread,
  }: {
    gameThread: GameThread;
    loaderThread: LoaderThread;
    renderThread: RenderThread;
  }) {
    this.#toGameThread = gameThread.postMessage.bind(gameThread);
    this.#toLoaderThread = loaderThread.postMessage.bind(loaderThread);
    this.#toRenderThread = renderThread.postMessage.bind(renderThread);

    loaderThread.onGltfLoaded = ({ id, scene }) => {
      // Set the loaded scene root to the glTF entity
      scene.entities = Object.values(scene.entities).filter((entityJSON) => {
        if (entityJSON.parentId === "root") entityJSON.parentId = id;
        if (entityJSON.id === "root") return false;
        return true;
      });

      // Add loaded glTF to the scene
      this.#scene.loadJSON(scene);

      // Send stripped down glTF to game thread
      const strippedScene: Partial<SceneJSON> = {
        entities: scene.entities,
      };

      this.#toGameThread({
        subject: "load_json",
        data: { scene: strippedScene },
      });

      // Send glTF + transfer data to render thread
      const bitmaps = Object.values(scene.images).map((image) => image.bitmap);
      const accessors = Object.values(scene.accessors).map(
        (accessor) => accessor.array.buffer
      );
      const transfer = [...bitmaps, ...accessors];

      this.#toRenderThread(
        {
          subject: "load_json",
          data: { scene },
        },
        transfer
      );
    };
  }

  onmessage = (event: MessageEvent<SceneMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "load_json":
        this.loadJSON(data.scene);
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
        this.updateGlobalTransform(
          data.entityId,
          data.position,
          data.quaternion
        );
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

  get materials$() {
    return this.#scene.materials$;
  }

  get entities() {
    return this.#scene.entities;
  }

  set entities(entities: { [id: string]: Entity }) {
    this.#scene.entities = entities;
  }

  get materials() {
    return this.#scene.materials;
  }

  set materials(materials: { [id: string]: Material }) {
    this.#scene.materials = materials;
  }

  addEntity(entity: Entity) {
    this.#scene.addEntity(entity);

    const message: SceneMessage = {
      subject: "add_entity",
      data: { entity: entity.toJSON() },
    };

    this.#toGameThread(message);
    this.#toRenderThread(message);

    if (entity.mesh?.type === "glTF") {
      const uri = entity.mesh.uri;
      if (uri)
        this.#toLoaderThread({
          subject: "load_gltf",
          data: { id: entity.id, uri },
        });
    }
  }

  removeEntity(entityId: string) {
    this.#scene.removeEntity(entityId);

    const message: SceneMessage = {
      subject: "remove_entity",
      data: { entityId },
    };

    this.#toGameThread(message);
    this.#toRenderThread(message);
  }

  updateEntity(entityId: string, data: Partial<EntityJSON>) {
    this.#scene.updateEntity(entityId, data);

    const message: SceneMessage = {
      subject: "update_entity",
      data: { entityId, data },
    };

    this.#toGameThread(message);
    this.#toRenderThread(message);
  }

  updateGlobalTransform(entityId: string, position: Triplet, quaternion: Quad) {
    this.#scene.updateGlobalTransform(entityId, position, quaternion);

    const message: SceneMessage = {
      subject: "update_global_transform",
      data: {
        entityId,
        position,
        quaternion,
      },
    };

    this.#toGameThread(message);
  }

  addMaterial(material: Material) {
    this.#scene.addMaterial(material);

    this.#toRenderThread({
      subject: "add_material",
      data: { material: material.toJSON() },
    });
  }

  removeMaterial(materialId: string) {
    this.#scene.removeMaterial(materialId);

    this.#toRenderThread({ subject: "remove_material", data: { materialId } });
  }

  updateMaterial(materialId: string, data: Partial<MaterialJSON>) {
    this.#scene.updateMaterial(materialId, data);

    this.#toRenderThread({
      subject: "update_material",
      data: { materialId, data },
    });
  }

  toJSON(): SceneJSON {
    return this.#scene.toJSON();
  }

  loadJSON(json: Partial<SceneJSON>) {
    this.#scene.loadJSON(json);
  }

  destroy() {
    this.#scene.destroy();
  }
}

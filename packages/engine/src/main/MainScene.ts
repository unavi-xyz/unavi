import { GameThread } from "../game/GameThread";
import { ToGameMessage } from "../game/types";
import { LoaderThread } from "../loader/LoaderThread";
import { ToLoaderMessage } from "../loader/types";
import { RenderThread } from "../render/RenderThread";
import { FromRenderMessage, ToRenderMessage } from "../render/types";
import { Entity } from "../scene/Entity";
import { Material } from "../scene/Material";
import { Scene } from "../scene/Scene";
import {
  EntityJSON,
  MaterialJSON,
  SceneJSON,
  SceneMessage,
} from "../scene/types";
import { PostMessage } from "../types";

/*
 * Wrapper around the {@link Scene}.
 * Syncs state with the {@link RenderThread} and {@link GameThread}.
 */
export class MainScene {
  #toGameThread: PostMessage<ToGameMessage>;
  #toLoaderThread: PostMessage<ToLoaderMessage>;
  #toRenderThread: PostMessage<ToRenderMessage>;

  #scene = new Scene();

  #map = {
    loadedGltfUris: new Map<string, string | null>(),
    glTFRootEntities: new Map<string, string[]>(),
  };

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

    // Handle glTF loads
    loaderThread.onGltfLoaded = ({ id, scene }) => {
      // Remove previous loaded glTF entity
      const rootEntities = this.#map.glTFRootEntities.get(id);
      if (rootEntities) {
        rootEntities.forEach((entityId) => this.removeEntity(entityId));
        this.#map.glTFRootEntities.delete(id);
      }

      scene.entities = scene.entities.filter((entityJSON) => {
        // Filter out root entity
        if (entityJSON.id === "root") return false;

        // Add top level entities to the roots map
        if (entityJSON.parentId === "root") {
          entityJSON.parentId = id;
          const roots = this.#map.glTFRootEntities.get(id) ?? [];
          roots.push(entityJSON.id);
          this.#map.glTFRootEntities.set(id, roots);
        }

        return true;
      });

      // Add loaded glTF to the scene
      this.loadJSON(scene);
    };

    // Load glTFs when added to the scene
    this.#scene.entities$.subscribe((entities) => {
      Object.values(entities).forEach((entity) => {
        entity.mesh$.subscribe((mesh) => {
          if (mesh?.type === "glTF") {
            mesh.uri$.subscribe((uri) => {
              const loadedURI = this.#map.loadedGltfUris.get(entity.id);
              if (uri !== loadedURI) {
                this.#map.loadedGltfUris.set(entity.id, uri);

                if (uri === null) {
                  // Remove loaded glTF
                  const rootEntities = this.#map.glTFRootEntities.get(
                    entity.id
                  );
                  if (rootEntities) {
                    rootEntities.forEach((entityId) =>
                      this.removeEntity(entityId)
                    );
                    this.#map.glTFRootEntities.delete(entity.id);
                  }
                  return;
                }

                // Load glTF
                this.#toLoaderThread({
                  subject: "load_gltf",
                  data: {
                    id: entity.id,
                    uri,
                  },
                });
              }
            });
          }
        });
      });
    });
  }

  onmessage = (event: MessageEvent<FromRenderMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "set_transform": {
        this.#scene.updateEntity(data.entityId, {
          position: data.position,
          rotation: data.rotation,
          scale: data.scale,
        });
        break;
      }
      case "set_global_transform": {
        this.#toGameThread(event.data);
        break;
      }
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
    // Remove root entity
    json.entities = json.entities?.filter((entity) => entity.id !== "root");

    // Send stripped down scene to game thread
    const strippedScene: Partial<SceneJSON> = {
      entities: json.entities,
    };

    this.#toGameThread({
      subject: "load_json",
      data: { scene: strippedScene },
    });

    // Send full scene + transfer data to render thread
    const bitmaps = json.images
      ? Object.values(json.images).map((image) => image.bitmap)
      : [];
    const accessors = json.accessors
      ? Object.values(json.accessors).map((accessor) => accessor.array.buffer)
      : [];

    const transfer = [...bitmaps, ...accessors];

    this.#toRenderThread(
      {
        subject: "load_json",
        data: { scene: json },
      },
      transfer
    );

    // Load scene
    this.#scene.loadJSON(json);
  }

  destroy() {
    this.#scene.destroy();
  }
}

import { LoaderThread } from "../loader/LoaderThread";
import { ToLoaderMessage } from "../loader/types";
import { PhysicsThread } from "../physics/PhysicsThread";
import { ToPhysicsMessage } from "../physics/types";
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
 * Syncs state with the {@link RenderThread} and {@link PhysicsThread}.
 */
export class MainScene {
  #toPhysicsThread: PostMessage<ToPhysicsMessage>;
  #toLoaderThread: PostMessage<ToLoaderMessage>;
  #toRenderThread: PostMessage<ToRenderMessage>;

  #scene = new Scene();

  #map = {
    loadedGltfUris: new Map<string, string | null>(),
    glTFRootEntities: new Map<string, string[]>(),
  };

  #gltfLoadCallbacks: Array<(id: string) => void> = [];

  constructor({
    physicsThread,
    loaderThread,
    renderThread,
  }: {
    physicsThread: PhysicsThread;
    loaderThread: LoaderThread;
    renderThread: RenderThread;
  }) {
    this.#toPhysicsThread = physicsThread.postMessage.bind(physicsThread);
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

      // Call glTF load callbacks
      this.#gltfLoadCallbacks.forEach((callback) => callback(id));
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
        this.#toPhysicsThread(event.data);
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

  set entities(entities: typeof Scene.prototype.entities) {
    this.#scene.entities = entities;
  }

  get materials() {
    return this.#scene.materials;
  }

  set materials(materials: typeof Scene.prototype.materials) {
    this.#scene.materials = materials;
  }

  get images() {
    return this.#scene.images;
  }

  set images(images: typeof Scene.prototype.images) {
    this.#scene.images = images;
  }

  addEntity(entity: Entity) {
    this.#scene.addEntity(entity);

    const message: SceneMessage = {
      subject: "add_entity",
      data: { entity: entity.toJSON() },
    };

    this.#toPhysicsThread(message);
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

    this.#toPhysicsThread(message);
    this.#toRenderThread(message);
  }

  updateEntity(entityId: string, data: Partial<EntityJSON>) {
    this.#scene.updateEntity(entityId, data);

    const message: SceneMessage = {
      subject: "update_entity",
      data: { entityId, data },
    };

    this.#toPhysicsThread(message);
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

  toJSON(includeInternal?: boolean): SceneJSON {
    return this.#scene.toJSON(includeInternal);
  }

  async loadJSON(json: Partial<SceneJSON>) {
    // Remove root entity
    json.entities = json.entities?.filter((entity) => entity.id !== "root");

    // Only send entities to physics thread
    const strippedScene: Partial<SceneJSON> = {
      entities: json.entities,
    };

    this.#toPhysicsThread({
      subject: "load_json",
      data: { scene: strippedScene },
    });

    // Send full scene to render thread
    this.#toRenderThread({
      subject: "load_json",
      data: { scene: json },
    });

    // Load scene
    this.#scene.loadJSON(json);

    // Wait for all glTFs to load
    if (json.entities) {
      const glTFs = json.entities.filter(
        (entity) => entity.mesh?.type === "glTF"
      );

      await Promise.all(
        glTFs.map(
          (entity) =>
            new Promise<void>((resolve) => {
              const index = this.#gltfLoadCallbacks.push((id) => {
                if (id === entity.id) {
                  resolve();

                  // Remove callback
                  this.#gltfLoadCallbacks.splice(index, 1);
                }
              });
            })
        )
      );
    }
  }

  clear() {
    // Remove all entities
    Object.values(this.#scene.entities).forEach((entity) => {
      if (entity.parentId === "root") this.removeEntity(entity.id);
    });

    // Remove all materials
    Object.values(this.#scene.materials).forEach((material) =>
      this.removeMaterial(material.id)
    );
  }

  destroy() {
    this.#scene.destroy();
  }
}

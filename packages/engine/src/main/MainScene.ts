import { LoaderThread } from "../loader/LoaderThread";
import { ToLoaderMessage } from "../loader/types";
import { PhysicsThread } from "../physics/PhysicsThread";
import { ToPhysicsMessage } from "../physics/types";
import { RenderThread } from "../render/RenderThread";
import { FromRenderMessage, ToRenderMessage } from "../render/types";
import { Mesh, MeshJSON } from "../scene";
import { Material } from "../scene/Material";
import { Node } from "../scene/Node";
import { Scene } from "../scene/Scene";
import {
  MaterialJSON,
  NodeJSON,
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

  #gltfLoadCallbacks = new Map<string, () => void>();

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
      // Remove previous loaded glTF node
      const rootEntities = this.#map.glTFRootEntities.get(id);
      if (rootEntities) {
        rootEntities.forEach((nodeId) => this.removeNode(nodeId));
        this.#map.glTFRootEntities.delete(id);
      }

      scene.entities = scene.entities.filter((entityJSON) => {
        // Filter out root node
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
      const callback = this.#gltfLoadCallbacks.get(id);
      if (callback) callback();
    };

    // Load glTFs when added to the scene
    this.#scene.nodes$.subscribe((entities) => {
      Object.values(entities).forEach((node) => {
        node.meshId$.subscribe((meshId) => {
          if (!meshId) return;
          const mesh = this.#scene.meshes[meshId];
          if (mesh?.type === "glTF") {
            mesh.uri$.subscribe((uri) => {
              const loadedURI = this.#map.loadedGltfUris.get(node.id);
              if (uri !== loadedURI) {
                this.#map.loadedGltfUris.set(node.id, uri);

                if (uri === null) {
                  // Remove loaded glTF
                  const rootEntities = this.#map.glTFRootEntities.get(node.id);
                  if (rootEntities) {
                    rootEntities.forEach((nodeId) => this.removeNode(nodeId));
                    this.#map.glTFRootEntities.delete(node.id);
                  }
                  return;
                }

                // Load glTF
                this.#toLoaderThread({
                  subject: "load_gltf",
                  data: {
                    id: node.id,
                    uri,
                  },
                });
              }
            });
          }
        });
      });
    });

    // Handle spawn changes
    this.#scene.spawn$.subscribe((spawn) => {
      this.#toRenderThread({
        subject: "load_json",
        data: {
          scene: {
            spawn,
          },
        },
      });

      this.#toPhysicsThread({
        subject: "load_json",
        data: {
          scene: {
            spawn,
          },
        },
      });
    });
  }

  onmessage = (event: MessageEvent<FromRenderMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "set_transform": {
        this.#scene.updateNode(data.nodeId, {
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

  get spawn() {
    return this.#scene.spawn;
  }

  get spawn$() {
    return this.#scene.spawn$;
  }

  get entities() {
    return this.#scene.nodes;
  }

  get entities$() {
    return this.#scene.nodes$;
  }

  get meshes() {
    return this.#scene.meshes;
  }

  get meshes$() {
    return this.#scene.meshes$;
  }

  get materials() {
    return this.#scene.materials;
  }

  get materials$() {
    return this.#scene.materials$;
  }

  get images() {
    return this.#scene.images;
  }

  addNode(node: Node) {
    this.#scene.addNode(node);

    const message: SceneMessage = {
      subject: "add_node",
      data: { node: node.toJSON() },
    };

    this.#toPhysicsThread(message);
    this.#toRenderThread(message);
  }

  updateNode(nodeId: string, data: Partial<NodeJSON>) {
    this.#scene.updateNode(nodeId, data);

    const message: SceneMessage = {
      subject: "update_node",
      data: { nodeId, data },
    };

    this.#toPhysicsThread(message);
    this.#toRenderThread(message);
  }

  removeNode(nodeId: string) {
    this.#scene.removeNode(nodeId);

    const message: SceneMessage = {
      subject: "remove_node",
      data: { nodeId },
    };

    this.#toPhysicsThread(message);
    this.#toRenderThread(message);
  }

  addMesh(mesh: Mesh) {
    if (mesh?.type === "glTF") {
      const uri = mesh.uri;
      if (uri)
        this.#toLoaderThread({
          subject: "load_gltf",
          data: { id: mesh.id, uri },
        });
    }

    this.#scene.addMesh(mesh);

    const message: SceneMessage = {
      subject: "add_mesh",
      data: { mesh: mesh.toJSON() },
    };

    this.#toPhysicsThread(message);
    this.#toRenderThread(message);
  }

  updateMesh(meshId: string, data: Partial<MeshJSON>) {
    this.#scene.updateMesh(meshId, data);

    const message: SceneMessage = {
      subject: "update_mesh",
      data: { meshId, data },
    };

    this.#toPhysicsThread(message);
    this.#toRenderThread(message);
  }

  removeMesh(meshId: string) {
    this.#scene.removeMesh(meshId);

    const message: SceneMessage = {
      subject: "remove_mesh",
      data: { meshId },
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
    // Remove root node
    json.entities = json.entities?.filter((node) => node.id !== "root");

    // Send stripped down scene to physics thread
    const strippedScene: Partial<SceneJSON> = {
      spawn: json.spawn,
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
    if (json.meshes) {
      const glTFs = json.meshes.filter((mesh) => mesh?.type === "glTF");

      await Promise.all(
        glTFs.map((node) => {
          return new Promise<void>((resolve) => {
            this.#gltfLoadCallbacks.set(node.id, () => {
              this.#gltfLoadCallbacks.delete(node.id);
              resolve();
            });
          });
        })
      );
    }
  }

  clear() {
    this.#scene.spawn = [0, 0, 0];

    // Remove all entities
    Object.values(this.#scene.nodes).forEach((node) => {
      if (node.parentId === "root") this.removeNode(node.id);
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

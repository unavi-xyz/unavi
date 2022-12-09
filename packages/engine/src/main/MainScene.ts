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
import { MaterialJSON, NodeJSON, SceneJSON, SceneMessage } from "../scene/types";
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
  };

  #gltfLoadSpawn = new Map<string, boolean>();
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
    loaderThread.onGltfLoaded = async ({ id, scene }) => {
      const parentNode = Object.values(this.#scene.nodes).find((node) => node.meshId === id);
      if (!parentNode) return;

      // Attach nodes to parent
      scene.nodes = scene.nodes.filter((nodeJSON) => {
        // Filter out root node
        if (nodeJSON.id === "root") return false;
        if (nodeJSON.parentId === "root") nodeJSON.parentId = parentNode.id;
        return true;
      });

      // Add loaded glTF to the scene
      const loadSpawn = this.#gltfLoadSpawn.get(id);
      await this.loadJSON(scene, false, loadSpawn);

      // Call glTF load callbacks
      const callback = this.#gltfLoadCallbacks.get(id);
      if (callback) callback();
    };

    // Load glTFs when added to the scene
    this.#scene.meshes$.subscribe((meshes) => {
      Object.values(meshes).forEach((mesh) => {
        if (mesh.type === "glTF") {
          mesh.uri$.subscribe((uri) => {
            const loadedURI = this.#map.loadedGltfUris.get(mesh.id);
            if (uri && uri !== loadedURI) {
              this.#map.loadedGltfUris.set(mesh.id, uri);

              // Load glTF
              this.#toLoaderThread({
                subject: "load_gltf",
                data: {
                  id: mesh.id,
                  uri,
                },
              });
            }
          });
        }
      });
    });

    // Handle spawn changes
    this.#scene.spawnId$.subscribe((spawnId) => {
      this.#toRenderThread({
        subject: "load_json",
        data: { scene: { spawnId } },
      });

      this.#toPhysicsThread({
        subject: "load_json",
        data: { scene: { spawnId } },
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

  get spawnId() {
    return this.#scene.spawnId;
  }

  set spawnId(spawnId: string | null) {
    this.#scene.spawnId = spawnId;
  }

  get spawnId$() {
    return this.#scene.spawnId$;
  }

  get accessors() {
    return this.#scene.accessors;
  }

  get animations() {
    return this.#scene.animations;
  }

  get nodes() {
    return this.#scene.nodes;
  }

  get nodes$() {
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

  removeImage(imageId: string) {
    this.#scene.removeImage(imageId);
  }

  toJSON(includeInternal?: boolean): SceneJSON {
    return this.#scene.toJSON(includeInternal);
  }

  async loadJSON(json: Partial<SceneJSON>, loadGltfSpawn = false, loadSpawn = true) {
    if (!loadSpawn) delete json.spawnId;

    // Remove root node
    json.nodes = json.nodes?.filter((node) => node.id !== "root");

    // Send stripped down scene to physics thread
    const strippedScene: Partial<SceneJSON> = {
      spawnId: json.spawnId,
      nodes: json.nodes,
      meshes: json.meshes,
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
      await Promise.all(
        json.meshes.map((mesh) => {
          if (mesh.type !== "glTF" || !mesh.uri) return;

          return new Promise<void>((resolve) => {
            this.#gltfLoadSpawn.set(mesh.id, loadGltfSpawn);
            this.#gltfLoadCallbacks.set(mesh.id, () => {
              this.#gltfLoadSpawn.delete(mesh.id);
              this.#gltfLoadCallbacks.delete(mesh.id);
              resolve();
            });
          });
        })
      );
    }
  }

  clear() {
    // Remove spawn
    this.#scene.spawnId = null;
    this.#toPhysicsThread({
      subject: "load_json",
      data: { scene: { spawnId: null } },
    });
    this.#toRenderThread({
      subject: "load_json",
      data: { scene: { spawnId: null } },
    });

    // Remove all nodes
    Object.values(this.#scene.nodes).forEach((node) => {
      if (node.parentId === "root") this.removeNode(node.id);
    });

    // Remove all materials
    Object.values(this.#scene.materials).forEach((material) => this.removeMaterial(material.id));
  }

  destroy() {
    this.#scene.destroy();
  }
}

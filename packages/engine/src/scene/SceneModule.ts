import {
  Accessor,
  Document,
  ExtensionProperty,
  GLTF,
  Mesh,
  Node,
  Primitive,
  WebIO,
} from "@gltf-transform/core";
import { KHRDracoMeshCompression } from "@gltf-transform/extensions";
import { draco } from "@gltf-transform/functions";
import {
  AvatarExtension,
  BehaviorExtension,
  Collider,
  ColliderExtension,
  SpawnPointExtension,
} from "@wired-labs/gltf-extensions";

import { Engine } from "../Engine";
import { extensions } from "../extensions";
import { PhysicsModule } from "../physics/PhysicsModule";
import { optimizeDocument } from "../render";
import { RenderModule } from "../render/RenderModule";
import { getCustomMeshData } from "../render/scene/utils/getCustomMeshData";
import { subscribe } from "../utils/subscribe";
import { AnimationJSON } from "./attributes/Animations";
import { MaterialJSON } from "./attributes/Materials";
import { MeshExtras, MeshJSON } from "./attributes/Meshes";
import { NodeJSON } from "./attributes/Nodes";
import { PrimitiveJSON } from "./attributes/Primitives";
import { SceneMessage } from "./messages";
import { Scene } from "./Scene";

/**
 * Handles scene related logic for the main thread.
 * Syncs changes to the scene with the worker threads.
 *
 * @group Modules
 */
export class SceneModule extends Scene {
  #render: RenderModule;
  #physics: PhysicsModule;

  constructor(engine: Engine) {
    super();

    this.#render = engine.render;
    this.#physics = engine.physics;

    this.accessor.addEventListener("create", ({ data }) => {
      const accessor = this.accessor.store.get(data.id);
      if (!accessor) throw new Error("Accessor not found");
      this.#onAccessorCreate(accessor);
    });

    this.primitive.addEventListener("create", ({ data }) => {
      const primitive = this.primitive.store.get(data.id);
      if (!primitive) throw new Error("Primitive not found");
      this.#onPrimitiveCreate(primitive);
    });

    this.mesh.addEventListener("create", ({ data }) => {
      const mesh = this.mesh.store.get(data.id);
      if (!mesh) throw new Error("Mesh not found");
      this.#onMeshCreate(mesh);
    });

    this.node.addEventListener("create", ({ data }) => {
      const node = this.node.store.get(data.id);
      if (!node) throw new Error("Node not found");

      this.#onNodeCreate(node);

      const id = this.node.getId(node);
      if (!id) throw new Error("Id not found");

      const json = this.node.toJSON(node);
      this.#publish({ subject: "change_node", data: { id, json } });
    });
  }

  async #createIO() {
    return new WebIO().registerExtensions(extensions).registerDependencies({
      // DracoDecoderModule is a global variable, needs to be set by the user of this library
      // @ts-ignore
      "draco3d.decoder": await new DracoDecoderModule(),
    });
  }

  async export({ log = false, optimize = false } = {}) {
    const io = await this.#createIO();

    // Merge all buffers into one
    const buffers = this.doc.getRoot().listBuffers();
    buffers.forEach((buffer) => buffer.dispose());

    const buffer = this.doc.createBuffer();

    const accessors = this.doc.getRoot().listAccessors();
    accessors.forEach((accessor) => accessor.setBuffer(buffer));

    this.doc
      .getRoot()
      .listExtensionsUsed()
      .forEach((extension) => {
        // Remove extension if it's empty
        if (extension.listProperties().length === 0) {
          extension.dispose();
        }

        // Remove draco compression
        if (extension.extensionName === KHRDracoMeshCompression.EXTENSION_NAME) {
          extension.dispose();
        }
      });

    if (log) console.info("Exporting:", await io.writeJSON(this.doc));

    if (optimize) {
      // Clone document to avoid modifying the original
      const optimizedDoc = this.doc.clone();

      // Optimize model
      await optimizeDocument(optimizedDoc);

      // Compress model
      try {
        // Create another clone to avoid modifying the optimized model
        // Sometimes compression fails, so we want to return the optimized model in that case
        const compressedDoc = optimizedDoc.clone();

        await io.registerDependencies({
          // @ts-ignore
          "draco3d.encoder": await new DracoEncoderModule(),
        });

        await compressedDoc.transform(draco());

        // Return the compressed model
        return await io.writeBinary(compressedDoc);
      } catch (err) {
        console.warn("Failed to compress model", err);
      }

      // Return the optimized model
      return await io.writeBinary(optimizedDoc);
    }

    return await io.writeBinary(this.doc);
  }

  async addBinary(array: Uint8Array) {
    const io = await this.#createIO();
    const doc = await io.readBinary(array);
    await this.addDocument(doc);
  }

  async addFiles(files: File[]) {
    const root = files.find((file) => file.name.endsWith(".gltf"));
    if (!root) throw new Error("No root file found");

    // Replace uris with blob urls
    const rootText = await root.text();
    const rootJSON = JSON.parse(rootText) as GLTF.IGLTF;

    const urls = files.map((file) => URL.createObjectURL(file));
    const getRelativeUrl = (url: string | undefined) => url?.split("/").pop();

    // Buffers
    rootJSON.buffers?.forEach((buffer) => {
      const fileIndex = files.findIndex((file) => file.name === buffer.uri);
      if (fileIndex !== -1) buffer.uri = getRelativeUrl(urls[fileIndex]);
    });

    // Images
    rootJSON.images?.forEach((image) => {
      const fileIndex = files.findIndex((file) => file.name === image.uri);
      if (fileIndex !== -1) image.uri = getRelativeUrl(urls[fileIndex]);
    });

    // Load root file
    const newRoot = new File([JSON.stringify(rootJSON)], root.name);
    await this.addFile(newRoot);

    urls.forEach((url) => URL.revokeObjectURL(url));
  }

  async addFile(file: File) {
    const io = await this.#createIO();

    let doc: Document | undefined;

    if (file.name.endsWith(".glb")) {
      const buffer = await file.arrayBuffer();
      const array = new Uint8Array(buffer);
      doc = await io.readBinary(array);
    } else if (file.name.endsWith(".gltf")) {
      const url = URL.createObjectURL(file);
      doc = await io.read(url);
      URL.revokeObjectURL(url);
    }

    if (!doc) throw new Error("Invalid file");

    const scene = doc.getRoot().getDefaultScene();

    const name = file.name.split(".")[0];
    const groupNode = doc.createNode(name);

    // Add top level nodes to group node
    scene?.listChildren().forEach((node) => groupNode.addChild(node));
    scene?.addChild(groupNode);

    // Add physics colliders
    doc
      .getRoot()
      .listNodes()
      .forEach((node) => {
        if (!doc) throw new Error("Document not found");

        const mesh = node.getMesh();
        if (!mesh) return;

        const collider = node.getExtension<Collider>(ColliderExtension.EXTENSION_NAME);
        if (collider) return;

        const meshCollider = new Collider(doc.getGraph());
        meshCollider.setType("trimesh");
        meshCollider.setMesh(mesh);

        node.setExtension<Collider>(ColliderExtension.EXTENSION_NAME, meshCollider);
      });

    await this.addDocument(doc);
  }

  clear() {
    this.animation.store.forEach((animation) => animation.dispose());
    this.skin.store.forEach((skin) => skin.dispose());
    this.node.store.forEach((node) => node.dispose());
    this.mesh.store.forEach((mesh) => mesh.dispose());
    this.primitive.store.forEach((primitive) => primitive.dispose());
    this.accessor.store.forEach((accessor) => accessor.dispose());
    this.buffer.store.forEach((buffer) => buffer.dispose());
    this.texture.store.forEach((texture) => texture.dispose());
    this.material.store.forEach((material) => material.dispose());

    Object.values(this.extensions).forEach((extension) => extension.dispose());

    this.extensions = {
      avatar: this.doc.createExtension(AvatarExtension),
      behavior: this.doc.createExtension(BehaviorExtension),
      collider: this.doc.createExtension(ColliderExtension),
      spawn: this.doc.createExtension(SpawnPointExtension),
    };
  }

  override processChanges() {
    this.buffer.processChanges().forEach((buffer) => {
      const id = this.buffer.getId(buffer);
      if (!id) throw new Error("Id not found");
      const json = this.buffer.toJSON(buffer);

      this.#publish({ subject: "create_buffer", data: { id, json } });

      buffer.addEventListener("dispose", () => {
        this.#publish({ subject: "dispose_buffer", data: id });
      });
    });

    this.accessor.processChanges().forEach((accessor) => {
      this.#onAccessorCreate(accessor);
    });

    this.texture.processChanges().forEach((texture) => {
      const id = this.texture.getId(texture);
      if (!id) throw new Error("Id not found");
      const json = this.texture.toJSON(texture);

      this.#render.send({ subject: "create_texture", data: { id, json } });

      texture.addEventListener("dispose", () => {
        this.#render.send({ subject: "dispose_texture", data: id });
      });
    });

    this.material.processChanges().forEach((material) => {
      const id = this.material.getId(material);
      if (!id) throw new Error("Id not found");
      const json = this.material.toJSON(material);

      this.#render.send({ subject: "create_material", data: { id, json } });

      material.addEventListener("change", (e) => {
        const attribute = e.attribute as keyof MaterialJSON;
        const json = this.material.toJSON(material);
        const value = json[attribute];

        this.#render.send({
          subject: "change_material",
          data: { id, json: { [attribute]: value } },
        });
      });

      material.addEventListener("dispose", () => {
        this.#render.send({ subject: "dispose_material", data: id });
      });
    });

    this.primitive.processChanges().forEach((primitive) => {
      this.#onPrimitiveCreate(primitive);
    });

    this.mesh.processChanges().forEach((mesh) => {
      this.#onMeshCreate(mesh);
    });

    // Create nodes, then skins, then send node json
    // This is because skins and nodes reference each other
    const nodes = this.node.processChanges();
    const skins = this.skin.processChanges();

    nodes.forEach((node) => {
      this.#onNodeCreate(node);
    });

    skins.forEach((skin) => {
      const id = this.skin.getId(skin);
      if (!id) throw new Error("Id not found");

      const json = this.skin.toJSON(skin);
      this.#render.send({ subject: "create_skin", data: { id, json } });

      skin.addEventListener("dispose", () => {
        this.#render.send({ subject: "dispose_skin", data: id });
      });
    });

    nodes.forEach((node) => {
      const id = this.node.getId(node);
      if (!id) throw new Error("Id not found");

      const json = this.node.toJSON(node);
      this.#publish({ subject: "change_node", data: { id, json } });
    });

    this.animation.processChanges().forEach((animation) => {
      const id = this.animation.getId(animation);
      if (!id) throw new Error("Id not found");

      const json = this.animation.toJSON(animation);
      this.#publish({ subject: "create_animation", data: { id, json } });

      animation.addEventListener("change", (e) => {
        const attribute = e.attribute as keyof AnimationJSON;
        const json = this.animation.toJSON(animation);
        const value = json[attribute];

        this.#publish({ subject: "change_animation", data: { id, json: { [attribute]: value } } });
      });

      animation.addEventListener("dispose", () => {
        this.#publish({ subject: "dispose_animation", data: id });
      });
    });
  }

  #onAccessorCreate(accessor: Accessor) {
    const id = this.accessor.getId(accessor);
    if (!id) throw new Error("Id not found");

    const json = this.accessor.toJSON(accessor);
    this.#publish({ subject: "create_accessor", data: { id, json } });

    accessor.addEventListener("dispose", () => {
      this.#publish({ subject: "dispose_accessor", data: id });
    });
  }

  #onNodeCreate(node: Node) {
    const id = this.node.getId(node);
    if (!id) throw new Error("Id not found");

    this.#publish({ subject: "create_node", data: { id, json: {} } });

    let extensionListeners: Array<{ extension: ExtensionProperty; listener: () => void }> = [];

    const setupExtensionListeners = () => {
      // Remove old listeners
      extensionListeners.forEach(({ extension, listener }) => {
        extension.removeEventListener("change", listener);
      });

      // Add new listeners
      extensionListeners = node.listExtensions().map((extension) => {
        const listener = () => {
          const json = this.node.toJSON(node);
          const value = json.extensions;

          this.#publish({ subject: "change_node", data: { id, json: { extensions: value } } });
        };

        extension.addEventListener("change", listener);

        if (extension instanceof Collider) {
          if (extension.getType() === "trimesh") {
            // Set collider mesh
            const mesh = node.getMesh();
            extension.setMesh(mesh);
          }
        }

        return { extension, listener };
      });
    };

    setupExtensionListeners();

    node.addEventListener("change", (e) => {
      const attribute = e.attribute as keyof NodeJSON;
      const json = this.node.toJSON(node);
      const value = json[attribute];

      this.#publish({ subject: "change_node", data: { id, json: { [attribute]: value } } });

      if (attribute === "mesh") {
        // Update mesh collider
        const collider = node.getExtension<Collider>(ColliderExtension.EXTENSION_NAME);

        if (collider?.getType() === "trimesh") {
          const meshId = value as string | null;

          if (meshId) {
            const mesh = this.mesh.store.get(meshId);
            if (!mesh) throw new Error("Mesh not found");
            collider.setMesh(mesh);
          } else {
            collider.setMesh(null);
          }
        }
      }

      if (attribute === "extensions") {
        setupExtensionListeners();
      }
    });

    node.addEventListener("dispose", () => {
      this.#publish({ subject: "dispose_node", data: id });
    });
  }

  #onMeshCreate(mesh: Mesh) {
    const id = this.mesh.getId(mesh);
    if (!id) throw new Error("Id not found");

    const json = this.mesh.toJSON(mesh);
    this.#publish({ subject: "create_mesh", data: { id, json } });

    mesh.addEventListener("change", (e) => {
      const attribute = e.attribute as keyof MeshJSON;
      const json = this.mesh.toJSON(mesh);
      const value = json[attribute];

      this.#publish({ subject: "change_mesh", data: { id, json: { [attribute]: value } } });
    });

    mesh.addEventListener("dispose", () => {
      this.#publish({ subject: "dispose_mesh", data: id });
    });

    // Create custom mesh
    subscribe(mesh, "Extras", (extras: MeshExtras) => {
      if (!extras.customMesh) return;

      // Dispose primitives if custom mesh is used
      mesh.listPrimitives().forEach((primitive) => {
        // Dispose accessors if they are not used by other primitives
        primitive.listAttributes().forEach((accessor) => {
          if (accessor.listParents().length === 1) accessor.dispose();
        });

        const indices = primitive.getIndices();
        if (indices && indices.listParents().length === 1) indices.dispose();

        primitive.dispose();
      });

      // Create acessors
      const { positions, normals, indices } = getCustomMeshData(extras.customMesh);

      const { id: positionsId, object: position } = this.accessor.create({
        array: new Float32Array(positions),
        type: "VEC3",
        componentType: 5126,
      });

      const { id: normalsId, object: normal } = this.accessor.create({
        array: new Float32Array(normals),
        type: "VEC3",
        componentType: 5126,
      });

      const { id: indicesId, object: index } = this.accessor.create({
        array: new Uint16Array(indices),
        type: "SCALAR",
        componentType: 5121,
      });

      // Create new primitive
      const { object: primitive } = this.primitive.create({
        attributes: { POSITION: positionsId, NORMAL: normalsId },
        indices: indicesId,
      });

      // Add to mesh
      mesh.addPrimitive(primitive);

      return () => {
        // Dispose primitive
        mesh.removePrimitive(primitive);
        primitive.dispose();
        position.dispose();
        normal.dispose();
        index.dispose();
      };
    });
  }

  #onPrimitiveCreate(primitive: Primitive) {
    const id = this.primitive.getId(primitive);
    if (!id) throw new Error("Id not found");

    const json = this.primitive.toJSON(primitive);
    this.#publish({ subject: "create_primitive", data: { id, json } });

    primitive.addEventListener("change", (e) => {
      const attribute = e.attribute as keyof PrimitiveJSON;
      const json = this.primitive.toJSON(primitive);
      const value = json[attribute];

      this.#publish({ subject: "change_primitive", data: { id, json: { [attribute]: value } } });
    });

    primitive.addEventListener("dispose", () => {
      this.#publish({ subject: "dispose_primitive", data: id });
    });
  }

  #publish(message: SceneMessage) {
    this.#render.send(message);
    this.#physics.send(message);
  }
}

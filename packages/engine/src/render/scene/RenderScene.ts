import { Accessor, Mesh, Node, Primitive, Texture, TextureInfo } from "@gltf-transform/core";
import {
  BoxGeometry,
  BufferAttribute,
  BufferGeometry,
  CapsuleGeometry,
  CylinderGeometry,
  DoubleSide,
  FrontSide,
  Mesh as ThreeMesh,
  MeshBasicMaterial,
  MeshStandardMaterial,
  Object3D,
  SphereGeometry,
  sRGBEncoding,
} from "three";
import { CSM } from "three/examples/jsm/csm/CSM";
import { mergeBufferGeometries } from "three/examples/jsm/utils/BufferGeometryUtils";

import { Collider, ColliderExtension } from "../../gltf";
import { MeshJSON, NodeJSON } from "../../scene";
import { MaterialJSON } from "../../scene/attributes/Materials";
import { PrimitiveJSON } from "../../scene/attributes/Primitives";
import { TextureInfoJSON, TextureInfoUtils } from "../../scene/attributes/TextureInfoUtils";
import { SceneMessage } from "../../scene/messages";
import { Scene } from "../../scene/Scene";
import { createTexture } from "./createTexture";

type NodeId = string;
type MeshId = string;
type PrimitiveId = string;
type MaterialId = string;

const defaultMaterial = new MeshStandardMaterial();

/**
 * Receives scene updates from the main thread
 * and translates them into Three.js objects.
 */
export class RenderScene extends Scene {
  #csm: CSM | null = null;
  #toMainThread: (message: SceneMessage) => void;

  root = new Object3D();

  materialObjects = new Map<MaterialId, MeshStandardMaterial>();
  primitiveObjects = new Map<PrimitiveId, ThreeMesh>();
  meshObjects = new Map<MeshId, Object3D>();
  instancedMeshObjects = new Map<NodeId, Object3D>();
  nodeObjects = new Map<NodeId, Object3D>();
  colliderObjects = new Map<NodeId, ThreeMesh>();

  colliderMaterial = new MeshBasicMaterial({ color: "#000000", wireframe: true });
  visuals = new Object3D();
  #visualsEnabled = false;

  constructor(toMainThread: (message: SceneMessage) => void) {
    super();

    this.#toMainThread = toMainThread;

    this.accessor.addEventListener("create", ({ data }) => {
      const accessor = this.accessor.store.get(data.id);
      if (!accessor) throw new Error("Accessor not found");

      const json = this.accessor.toJSON(accessor);

      toMainThread({ subject: "create_accessor", data: { id: data.id, json } });

      accessor.addEventListener("dispose", () => {
        toMainThread({ subject: "dispose_accessor", data: data.id });
      });
    });

    this.primitive.addEventListener("create", ({ data }) => {
      const primitive = this.primitive.store.get(data.id);
      if (!primitive) throw new Error("Primitive not found");

      const json = this.primitive.toJSON(primitive);

      toMainThread({ subject: "create_primitive", data: { id: data.id, json } });

      primitive.addEventListener("dispose", () => {
        toMainThread({ subject: "dispose_primitive", data: data.id });
      });
    });
  }

  get visualsEnabled() {
    return this.#visualsEnabled;
  }

  set visualsEnabled(enabled: boolean) {
    this.#visualsEnabled = enabled;
    if (!enabled) this.visuals.clear();
  }

  setCSM(csm: CSM | null) {
    this.#csm?.dispose();

    this.#csm = csm;

    for (const material of this.materialObjects.values()) {
      this.#csm?.setupMaterial(material);
    }
  }

  onmessage({ subject, data }: SceneMessage) {
    switch (subject) {
      case "create_buffer": {
        this.buffer.create(data.json, data.id);
        break;
      }

      case "dispose_buffer": {
        const buffer = this.buffer.store.get(data);
        if (buffer) buffer.dispose();
        break;
      }

      case "create_accessor": {
        this.accessor.create(data.json, data.id);
        break;
      }

      case "dispose_accessor": {
        const accessor = this.accessor.store.get(data);
        if (accessor) accessor.dispose();
        break;
      }

      case "create_texture": {
        this.texture.create(data.json, data.id);
        break;
      }

      case "dispose_texture": {
        const texture = this.texture.store.get(data);
        if (texture) texture.dispose();
        break;
      }

      case "create_material": {
        const { object: material } = this.material.create(data.json, data.id);

        const object = new MeshStandardMaterial();
        this.materialObjects.set(data.id, object);

        this.#csm?.setupMaterial(object);

        material.addEventListener("dispose", () => {
          object.map?.dispose();
          object.normalMap?.dispose();
          object.metalnessMap?.dispose();
          object.roughnessMap?.dispose();
          object.aoMap?.dispose();
          object.emissiveMap?.dispose();
          object.dispose();

          this.materialObjects.delete(data.id);
        });

        this.updateMaterial(data.id, data.json);
        break;
      }

      case "change_material": {
        this.updateMaterial(data.id, data.json);

        const material = this.material.store.get(data.id);
        if (!material) throw new Error("Material not found");
        this.material.applyJSON(material, data.json);
        break;
      }

      case "dispose_material": {
        const material = this.material.store.get(data);
        if (material) material.dispose();
        break;
      }

      case "create_primitive": {
        const { object: primitive } = this.primitive.create(data.json, data.id);

        const object = new ThreeMesh();
        this.primitiveObjects.set(data.id, object);

        object.castShadow = true;
        object.receiveShadow = true;

        primitive.addEventListener("dispose", () => {
          object.geometry.dispose();
          this.meshObjects.delete(data.id);
        });

        this.updatePrimitive(data.id, data.json);
        break;
      }

      case "change_primitive": {
        this.updatePrimitive(data.id, data.json);

        const primitive = this.primitive.store.get(data.id);
        if (!primitive) throw new Error("Primitive not found");
        this.primitive.applyJSON(primitive, data.json);
        break;
      }

      case "dispose_primitive": {
        const primitive = this.primitive.store.get(data);
        if (primitive) primitive.dispose();
        break;
      }

      case "create_mesh": {
        const { object: mesh } = this.mesh.create(data.json, data.id);

        const object = new Object3D();
        this.meshObjects.set(data.id, object);

        mesh.addEventListener("dispose", () => {
          this.meshObjects.delete(data.id);
        });

        this.updateMesh(data.id, data.json);
        break;
      }

      case "change_mesh": {
        this.updateMesh(data.id, data.json);

        const mesh = this.mesh.store.get(data.id);
        if (!mesh) throw new Error("Mesh not found");
        this.mesh.applyJSON(mesh, data.json);
        break;
      }

      case "dispose_mesh": {
        const mesh = this.mesh.store.get(data);
        if (mesh) mesh.dispose();
        break;
      }

      case "create_node": {
        const { object: node } = this.node.create(data.json, data.id);

        const object = new Object3D();
        this.root.add(object);
        this.nodeObjects.set(data.id, object);

        node.addEventListener("dispose", () => {
          this.nodeObjects.delete(data.id);
        });

        this.updateNode(data.id, data.json);
        break;
      }

      case "change_node": {
        this.updateNode(data.id, data.json);
        break;
      }

      case "dispose_node": {
        const node = this.node.store.get(data);
        if (node) node.dispose();
        break;
      }
    }
  }

  updateNode(id: string, json: Partial<NodeJSON>) {
    const node = this.node.store.get(id);
    if (!node) throw new Error("Node not found");

    const object = this.nodeObjects.get(id);
    if (!object) throw new Error("Object not found");

    if (json.name) object.name = json.name;

    if (json.translation) object.position.fromArray(json.translation);
    if (json.rotation) object.quaternion.fromArray(json.rotation);
    if (json.scale) object.scale.fromArray(json.scale);

    if (json.mesh !== undefined) {
      // Remove old mesh
      object.children.forEach((child) => {
        const id = this.getMeshId(child) ?? this.getInstancedMeshNodeId(child);
        const isMesh = id !== null;
        if (isMesh) object.remove(child);
      });

      // Add new mesh
      if (json.mesh) {
        const mesh = this.mesh.store.get(json.mesh);
        if (!mesh) throw new Error("Mesh not found");

        const meshObject = this.meshObjects.get(json.mesh);
        if (!meshObject) throw new Error("Mesh object not found");

        const instanceCount = mesh.listParents().filter((p) => p instanceof Node).length;

        if (instanceCount > 1) {
          const instance = meshObject.clone();
          this.instancedMeshObjects.set(id, instance);
          object.add(instance);
        } else {
          object.add(meshObject);
        }
      }
    }

    if (json.children) {
      // Remove old children
      object.children.forEach((child) => {
        const nodeId = this.getNodeId(child);
        const isNode = nodeId !== null;
        if (isNode) object.remove(child);
      });

      // Add new children
      json.children.forEach((childId) => {
        const nodeObject = this.nodeObjects.get(childId);
        if (!nodeObject) throw new Error("Node object not found");

        object.add(nodeObject);
      });
    }

    // Apply node JSON
    this.node.applyJSON(node, json);

    if (json.mesh !== undefined || json.extensions?.OMI_collider !== undefined) {
      this.updateColliderVisual(id);
    }
  }

  updateMesh(id: string, json: Partial<MeshJSON>) {
    const mesh = this.mesh.store.get(id);
    if (!mesh) throw new Error("Mesh not found");

    const object = this.meshObjects.get(id);
    if (!object) throw new Error("Mesh object not found");

    if (json.primitives) {
      // Remove previous primitive objects
      mesh.listPrimitives().forEach((primitive) => {
        const primitiveId = this.primitive.getId(primitive);
        if (!primitiveId) throw new Error("Primitive not found");

        const primitiveObject = this.primitiveObjects.get(primitiveId);
        if (!primitiveObject) throw new Error("Primitive object not found");

        object.remove(primitiveObject);
      });

      // Add new primitive objects as children
      json.primitives.forEach((primitiveId) => {
        const primitiveObject = this.primitiveObjects.get(primitiveId);
        if (!primitiveObject) throw new Error("Primitive object not found");

        object.add(primitiveObject);
      });
    }

    if (json.extras) {
      if (json.extras.customMesh) {
        // Remove old primitives
        mesh.listPrimitives().forEach((primitive) => {
          const primitiveId = this.primitive.getId(primitive);
          if (!primitiveId) throw new Error("Primitive not found");

          const primitiveObject = this.primitiveObjects.get(primitiveId);
          if (!primitiveObject) throw new Error("Primitive object not found");

          object.remove(primitiveObject);
          this.primitiveObjects.delete(primitiveId);
          primitive.dispose();
        });

        // Create custom mesh object
        let customMesh: ThreeMesh | null = null;

        switch (json.extras.customMesh.type) {
          case "Box": {
            const { width, height, depth } = json.extras.customMesh;
            const geometry = new BoxGeometry(width, height, depth);
            customMesh = new ThreeMesh(geometry, defaultMaterial);
            break;
          }

          case "Sphere": {
            const { radius, widthSegments, heightSegments } = json.extras.customMesh;
            const geometry = new SphereGeometry(radius, widthSegments, heightSegments);
            customMesh = new ThreeMesh(geometry, defaultMaterial);
            break;
          }

          case "Cylinder": {
            const { radiusTop, radiusBottom, height, radialSegments } = json.extras.customMesh;
            const geometry = new CylinderGeometry(radiusTop, radiusBottom, height, radialSegments);
            customMesh = new ThreeMesh(geometry, defaultMaterial);
            break;
          }
        }

        if (customMesh) {
          // Create new primitive
          const positions = customMesh.geometry.getAttribute("position");
          const indices = customMesh.geometry.getIndex();
          if (!indices) throw new Error("Indices not found");

          const { id: positionsId } = this.accessor.create({
            array: new Float32Array(positions.array),
            type: "VEC3",
            componentType: 5126,
          });
          const { id: indicesId } = this.accessor.create({
            array: new Uint8Array(indices.array),
            type: "SCALAR",
            componentType: 5121,
          });

          const { id: primitiveId, object: primitive } = this.primitive.create({
            indices: indicesId,
            attributes: { POSITION: positionsId },
          });

          mesh.addPrimitive(primitive);
          this.primitiveObjects.set(primitiveId, customMesh);
          object.add(customMesh);

          // Update main thread
          this.#toMainThread({
            subject: "change_mesh",
            data: { id, json: { primitives: [primitiveId] } },
          });
        }
      }
    }

    mesh.listParents().forEach((parent) => {
      if (parent instanceof Node) {
        const nodeId = this.node.getId(parent);
        if (!nodeId) throw new Error("Node not found");

        this.updateColliderVisual(nodeId);
      }
    });
  }

  updatePrimitive(id: string, json: Partial<PrimitiveJSON>) {
    const primitive = this.primitive.store.get(id);
    if (!primitive) throw new Error("Primitive not found");

    const object = this.primitiveObjects.get(id);
    if (!object) throw new Error("Primitive object not found");

    if (json.material !== undefined) {
      // Remove previous material
      object.material = defaultMaterial;

      // Add new material
      if (json.material) {
        const materialObject = this.materialObjects.get(json.material);
        if (!materialObject) throw new Error("Material not found");

        object.material = materialObject;
      }
    }

    if (json.indices !== undefined) {
      if (!json.indices) {
        object.geometry.setIndex(null);
      } else {
        const accessor = this.accessor.store.get(json.indices);
        if (!accessor) throw new Error("Accessor not found");

        const attribute = accessorToAttribute(accessor);
        object.geometry.setIndex(attribute);
      }
    }

    if (json.attributes) {
      // Remove previous attributes
      Object.values(THREE_ATTRIBUTE_NAMES).forEach((name) => {
        object.geometry.deleteAttribute(name);
      });

      // Add new attributes
      Object.entries(json.attributes).forEach(([name, accessorId]) => {
        const threeName = THREE_ATTRIBUTE_NAMES[name as keyof typeof THREE_ATTRIBUTE_NAMES];

        if (accessorId === null) {
          object.geometry.deleteAttribute(threeName);
        } else {
          const accessor = this.accessor.store.get(accessorId);
          if (!accessor) throw new Error("Accessor not found");

          const attribute = accessorToAttribute(accessor);

          if (!attribute) object.geometry.deleteAttribute(threeName);
          else object.geometry.setAttribute(threeName, attribute);
        }
      });
    }

    if (json.targets) {
      // Remove previous morph attributes
      Object.values(THREE_ATTRIBUTE_NAMES).forEach((name) => {
        delete object.geometry.morphAttributes[name];
      });

      // Add new morph attributes
      json.targets.forEach((target) => {
        Object.entries(target.attributes).forEach(([name, accessorId]) => {
          const threeName = THREE_ATTRIBUTE_NAMES[name as keyof typeof THREE_ATTRIBUTE_NAMES];

          if (!accessorId) {
            object.geometry.deleteAttribute(threeName);
          } else {
            const accessor = this.accessor.store.get(accessorId);
            if (!accessor) throw new Error("Accessor not found");

            const attribute = accessorToAttribute(accessor);

            if (!attribute) delete object.geometry.morphAttributes[threeName];
            else {
              const morphAttributes = object.geometry.morphAttributes[threeName];
              if (!morphAttributes) object.geometry.morphAttributes[threeName] = [attribute];
              else morphAttributes.push(attribute);
            }
          }
        });
      });
    }
  }

  async updateMaterial(id: string, json: Partial<MaterialJSON>) {
    const material = this.material.store.get(id);
    if (!material) throw new Error("Material not found");

    const object = this.materialObjects.get(id);
    if (!object) throw new Error("Material object not found");

    if (json.doubleSided !== undefined) {
      object.side = json.doubleSided ? DoubleSide : FrontSide;
    }

    if (json.baseColorFactor) {
      object.color.fromArray(json.baseColorFactor);
    }

    const getTextureData = (
      textureId: string | null | undefined,
      textureInfo: TextureInfoJSON | null | undefined,
      currentTexture: Texture | null,
      currentInfo: TextureInfo | null
    ) => {
      if (!material) throw new Error("Material not found");

      const currentInfoJSON = currentInfo ? TextureInfoUtils.toJSON(currentInfo) : null;
      let texture: Texture | null = currentTexture;
      let info: TextureInfoJSON | null = currentInfoJSON;

      if (textureId !== undefined) {
        if (textureId === null) texture = null;
        else {
          const foundTexture = this.texture.store.get(textureId);
          if (!foundTexture) throw new Error("Texture not found");
          texture = foundTexture;
        }
      }

      if (textureInfo !== undefined) info = textureInfo;

      return { texture, info };
    };

    if (json.baseColorTexture !== undefined || json.baseColorTextureInfo !== undefined) {
      // Remove previous texture
      if (object.map) object.map.dispose();

      // Create new texture
      const { texture, info } = getTextureData(
        json.baseColorTexture,
        json.baseColorTextureInfo,
        material.getBaseColorTexture(),
        material.getBaseColorTextureInfo()
      );

      const textureObject = await createTexture(texture, info);
      object.map = textureObject;

      if (textureObject) textureObject.encoding = sRGBEncoding;
    }

    if (
      json.metallicRoughnessTexture !== undefined ||
      json.metallicRoughnessTextureInfo !== undefined
    ) {
      // Remove previous texture
      if (object.metalnessMap) object.metalnessMap.dispose();

      // Create new texture
      const { texture, info } = getTextureData(
        json.metallicRoughnessTexture,
        json.metallicRoughnessTextureInfo,
        material.getMetallicRoughnessTexture(),
        material.getMetallicRoughnessTextureInfo()
      );

      const textureObject = await createTexture(texture, info);
      object.metalnessMap = textureObject;
      object.roughnessMap = textureObject;
    }

    if (json.metallicFactor !== undefined) {
      object.metalness = json.metallicFactor;
    }

    if (json.roughnessFactor !== undefined) {
      object.roughness = json.roughnessFactor;
    }

    if (json.normalTexture !== undefined || json.normalTextureInfo !== undefined) {
      // Remove previous texture
      if (object.normalMap) object.normalMap.dispose();

      // Create new texture
      const { texture, info } = getTextureData(
        json.normalTexture,
        json.normalTextureInfo,
        material.getNormalTexture(),
        material.getNormalTextureInfo()
      );

      const textureObject = await createTexture(texture, info);
      object.normalMap = textureObject;
    }

    if (json.normalScale !== undefined) {
      object.normalScale.set(json.normalScale, json.normalScale);
    }

    if (json.occlusionTexture !== undefined || json.occlusionTextureInfo !== undefined) {
      // Remove previous texture
      if (object.aoMap) object.aoMap.dispose();

      // Create new texture
      const { texture, info } = getTextureData(
        json.occlusionTexture,
        json.occlusionTextureInfo,
        material.getOcclusionTexture(),
        material.getOcclusionTextureInfo()
      );

      const textureObject = await createTexture(texture, info);
      object.aoMap = textureObject;
    }

    if (json.occlusionStrength !== undefined) {
      object.aoMapIntensity = json.occlusionStrength;
    }

    if (json.emissiveFactor !== undefined) {
      object.emissive.fromArray(json.emissiveFactor);
    }

    if (json.emissiveTexture !== undefined || json.emissiveTextureInfo !== undefined) {
      // Remove previous texture
      if (object.emissiveMap) object.emissiveMap.dispose();

      // Create new texture
      const { texture, info } = getTextureData(
        json.emissiveTexture,
        json.emissiveTextureInfo,
        material.getEmissiveTexture(),
        material.getEmissiveTextureInfo()
      );

      const textureObject = await createTexture(texture, info);
      object.emissiveMap = textureObject;

      if (textureObject) textureObject.encoding = sRGBEncoding;
    }

    if (json.alphaMode !== undefined || json.alphaCutoff !== undefined) {
      const cutoff = json.alphaCutoff !== undefined ? json.alphaCutoff : material.getAlphaCutoff();

      switch (json.alphaMode) {
        case "OPAQUE": {
          object.transparent = false;
          object.depthWrite = true;
          object.alphaTest = 0;
          break;
        }

        case "MASK": {
          object.transparent = false;
          object.depthWrite = true;
          object.alphaTest = cutoff;
          break;
        }

        case "BLEND": {
          object.transparent = true;
          object.depthWrite = false;
          object.alphaTest = 0;
          break;
        }
      }
    }

    object.needsUpdate = true;
  }

  updateColliderVisual(nodeId: string) {
    const node = this.node.store.get(nodeId);
    if (!node) throw new Error("Node not found");

    const object = this.nodeObjects.get(nodeId);
    if (!object) throw new Error("Object not found");

    const collider = node.getExtension<Collider>(ColliderExtension.EXTENSION_NAME);

    // Remove old collider
    const prevCollider = this.colliderObjects.get(nodeId);
    if (prevCollider) {
      object.remove(prevCollider);
      prevCollider.geometry.dispose();
      this.colliderObjects.delete(nodeId);
    }

    if (collider) {
      // Add new collider
      const meshId = collider.mesh instanceof Mesh ? this.mesh.getId(collider.mesh) : collider.mesh;
      if (meshId === undefined) throw new Error("Mesh not found");

      let colliderGeometry: BufferGeometry | null = null;

      switch (collider.type) {
        case "box": {
          if (!collider.size) break;
          colliderGeometry = new BoxGeometry(collider.size[0], collider.size[1], collider.size[2]);
          break;
        }

        case "sphere": {
          if (!collider.radius) break;
          colliderGeometry = new SphereGeometry(collider.radius);
          break;
        }

        case "capsule": {
          if (!collider.radius || !collider.height) break;
          colliderGeometry = new CapsuleGeometry(collider.radius, collider.height);
          break;
        }

        case "cylinder": {
          if (!collider.radius || !collider.height) break;
          colliderGeometry = new CylinderGeometry(
            collider.radius,
            collider.radius,
            collider.height
          );
          break;
        }

        case "trimesh": {
          if (!meshId) break;

          const mesh = this.mesh.store.get(meshId);
          if (!mesh) throw new Error("Mesh not found");

          const primitiveGeometries = mesh.listPrimitives().map((primitive) => {
            const primitiveId = this.primitive.getId(primitive);
            if (!primitiveId) throw new Error("Primitive not found");

            const primitiveObject = this.primitiveObjects.get(primitiveId);
            if (!primitiveObject) throw new Error("Primitive object not found");

            return primitiveObject.geometry;
          });

          if (primitiveGeometries.length === 0) break;

          colliderGeometry = mergeBufferGeometries(primitiveGeometries);
          break;
        }
      }

      if (colliderGeometry) {
        const colliderObject = new ThreeMesh(colliderGeometry, this.colliderMaterial);
        object.add(colliderObject);

        this.colliderObjects.set(nodeId, colliderObject);
      }
    }
  }

  getInstancedMeshNodeId(object: Object3D): string | null {
    for (const [id, clonedMeshObject] of this.instancedMeshObjects.entries()) {
      let found = false;
      clonedMeshObject.traverse((child) => {
        if (child === object) found = true;
      });
      if (found) return id;
    }

    return null;
  }

  getPrimitiveId(object: Object3D): string | null {
    for (const [id, primitiveObject] of this.primitiveObjects.entries()) {
      if (primitiveObject === object) return id;
    }

    return null;
  }

  getPrimitiveMeshId(primitive: Primitive): string | null {
    for (const [id, mesh] of this.mesh.store.entries()) {
      if (mesh.listPrimitives().includes(primitive)) return id;
    }

    return null;
  }

  getMeshId(object: Object3D): string | null {
    for (const [id, meshObject] of this.meshObjects.entries()) {
      if (meshObject === object) return id;
    }

    return null;
  }

  getMeshNodeId(mesh: Mesh): string | null {
    for (const [id, node] of this.node.store.entries()) {
      if (node.getMesh() === mesh) return id;
    }

    return null;
  }

  getNodeId(object: Object3D): string | null {
    for (const [id, nodeObject] of this.nodeObjects.entries()) {
      if (nodeObject === object) return id;
    }

    return null;
  }

  getObjectNodeId(object: Object3D): string | null {
    // Check primitives
    const primitiveId = this.getPrimitiveId(object);
    if (primitiveId) {
      const primitive = this.primitive.store.get(primitiveId);
      if (!primitive) throw new Error("Primitive not found");

      const meshId = this.getPrimitiveMeshId(primitive);
      if (!meshId) return null;

      const mesh = this.mesh.store.get(meshId);
      if (!mesh) throw new Error("Mesh not found");

      const nodeId = this.getMeshNodeId(mesh);
      if (!nodeId) return null;

      return nodeId;
    }

    // Check cloned meshes
    const clonedMeshId = this.getInstancedMeshNodeId(object);
    if (clonedMeshId) return clonedMeshId;

    return null;
  }
}

const THREE_ATTRIBUTE_NAMES = {
  POSITION: "position",
  NORMAL: "normal",
  TANGENT: "tangent",
  TEXCOORD_0: "uv",
  TEXCOORD_1: "uv2",
  COLOR_0: "color",
  JOINTS_0: "skinIndex",
  WEIGHTS_0: "skinWeight",
};

function accessorToAttribute(accessor: Accessor): BufferAttribute | null {
  const array = accessor.getArray();
  if (!array) return null;

  return new BufferAttribute(array, accessor.getElementSize(), accessor.getNormalized());
}

import { Accessor, Mesh, Primitive, Texture, TextureInfo } from "@gltf-transform/core";
import {
  BufferAttribute,
  DoubleSide,
  FrontSide,
  Mesh as ThreeMesh,
  MeshStandardMaterial,
  Object3D,
  sRGBEncoding,
} from "three";
import { CSM } from "three/examples/jsm/csm/CSM";
import { BoxGeometry, CylinderGeometry, SphereGeometry } from "three/src/Three";

import { MeshJSON } from "../scene";
import { SceneMessage } from "../scene/messages";
import { Scene } from "../scene/Scene";
import { MaterialJSON } from "../scene/utils/MaterialUtils";
import { NodeJSON } from "../scene/utils/NodeUtils";
import { PrimitiveJSON } from "../scene/utils/PrimitiveUtils";
import { TextureInfoJSON, TextureInfoUtils } from "../scene/utils/TextureInfoUtils";
import { createTexture } from "./createTexture";

const defaultMaterial = new MeshStandardMaterial();

/**
 * Receives scene updates from the main thread
 * and translates them into Three.js objects.
 */
export class RenderScene extends Scene {
  #csm: CSM | null = null;

  root = new Object3D();

  materialObjects = new Map<string, MeshStandardMaterial>();
  primitiveObjects = new Map<string, ThreeMesh>();
  customMeshObjects = new Map<string, ThreeMesh>();
  meshObjects = new Map<string, Object3D>();
  nodeObjects = new Map<string, Object3D>();

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
    if (!object) throw new Error("Node object not found");

    if (json.translation) object.position.fromArray(json.translation);
    if (json.rotation) object.quaternion.fromArray(json.rotation);
    if (json.scale) object.scale.fromArray(json.scale);

    if (json.mesh !== undefined) {
      // Remove previous mesh
      const prevMesh = node.getMesh();

      if (prevMesh) {
        const prevMeshId = this.mesh.getId(prevMesh);
        if (!prevMeshId) throw new Error("Mesh not found");

        const prevMeshObject = this.meshObjects.get(prevMeshId);
        if (!prevMeshObject) throw new Error("Mesh object not found");

        object.remove(prevMeshObject);
      }

      // Add new mesh
      if (json.mesh !== null) {
        const mesh = this.meshObjects.get(json.mesh);
        if (!mesh) throw new Error("Mesh not found");
        object.add(mesh);
      }
    }

    if (json.children) {
      // Remove previous children
      node.listChildren().forEach((child) => {
        const childId = this.node.getId(child);
        if (!childId) throw new Error("Child not found");

        const childObject = this.nodeObjects.get(childId);
        if (!childObject) throw new Error("Child object not found");

        object.remove(childObject);
      });

      // Add new children
      json.children.forEach((child) => {
        const childObject = this.nodeObjects.get(child);
        if (!childObject) throw new Error("Child not found");
        object.add(childObject);
      });
    }

    // Add to root if no parent
    if (object.parent === null) {
      this.root.add(object);
    }

    // Apply JSON after updating the object
    this.node.applyJSON(node, json);
  }

  updateMesh(id: string, json: Partial<MeshJSON>) {
    const mesh = this.mesh.store.get(id);
    if (!mesh) throw new Error("Mesh not found");

    const object = this.meshObjects.get(id);
    if (!object) throw new Error("Mesh object not found");

    if (json.extras) {
      // Remove previous custom mesh
      const prevCustomMesh = this.customMeshObjects.get(id);
      if (prevCustomMesh) {
        object.remove(prevCustomMesh);
        this.customMeshObjects.delete(id);
      }

      // Create custom mesh
      if (json.extras.customMesh) {
        let customMesh: ThreeMesh | undefined;

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
          object.add(customMesh);
          this.customMeshObjects.set(id, customMesh);
        }
      }
    }

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

    // Apply JSON after updating the object
    this.mesh.applyJSON(mesh, json);
  }

  updatePrimitive(id: string, json: Partial<PrimitiveJSON>) {
    const primitive = this.primitive.store.get(id);
    if (!primitive) throw new Error("Primitive not found");

    const object = this.primitiveObjects.get(id);
    if (!object) throw new Error("Primitive object not found");

    if (json.material) {
      const material = this.materialObjects.get(json.material);
      if (!material) throw new Error("Material not found");
      object.material = material;
    }

    if (json.indices !== undefined) {
      if (json.indices === null) {
        object.geometry.setIndex(null);
      } else {
        const accessor = this.accessor.store.get(json.indices);
        if (!accessor) throw new Error("Accessor not found");

        const attribute = accessorToAttribute(accessor);
        object.geometry.setIndex(attribute);
      }
    }

    if (json.attributes) {
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
      json.targets.forEach((target) => {
        Object.entries(target.attributes).forEach(([name, accessorId]) => {
          const threeName = THREE_ATTRIBUTE_NAMES[name as keyof typeof THREE_ATTRIBUTE_NAMES];

          if (accessorId === null) {
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

    // Apply JSON after updating the object
    this.primitive.applyJSON(primitive, json);
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

    // Apply JSON after updating the object
    this.material.applyJSON(material, json);
  }

  getCustomMeshObjectId(object: Object3D): string | null {
    for (const [id, customMeshObject] of this.customMeshObjects.entries()) {
      if (customMeshObject === object) return id;
    }

    return null;
  }

  getPrimitiveObjectId(object: Object3D): string | null {
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

  getMeshObjectId(object: Object3D): string | null {
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

  getNodeObjectId(object: Object3D): string | null {
    for (const [id, nodeObject] of this.nodeObjects.entries()) {
      if (nodeObject === object) return id;
    }

    return null;
  }

  getObjectNodeId(object: Object3D): string | null {
    const primitiveId = this.getPrimitiveObjectId(object);

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

    const meshId = this.getCustomMeshObjectId(object);

    if (meshId) {
      const mesh = this.mesh.store.get(meshId);
      if (!mesh) throw new Error("Mesh not found");

      const nodeId = this.getMeshNodeId(mesh);
      if (!nodeId) return null;

      return nodeId;
    }

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

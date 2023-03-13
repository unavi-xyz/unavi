import { Mesh, Primitive } from "@gltf-transform/core";
import { AnimationAction, AnimationMixer, Object3D } from "three";
import { CSM } from "three/examples/jsm/csm/CSM";

import { AccessorJSON } from "../../scene";
import { SceneMessage } from "../../scene/messages";
import { Scene } from "../../scene/Scene";
import { deepDispose } from "../utils/deepDispose";
import { AnimationBuilder } from "./builders/AnimationBuilder";
import { MaterialBuilder } from "./builders/MaterialBuilder";
import { MeshBuilder } from "./builders/MeshBuilder";
import { NodeBuilder } from "./builders/NodeBuilder";
import { PrimitiveBuilder } from "./builders/PrimitiveBuilder";
import { SkinBuilder } from "./builders/SkinBuilder";
import { TextureBuilder } from "./builders/TextureBuilder";

/**
 * Instants of a {@link Scene} for the render thread.
 * This class is responsible for creating and disposing Three.js objects.
 */
export class RenderScene extends Scene {
  #csm: CSM | null = null;
  #toMainThread: (message: SceneMessage) => void;

  root = new Object3D();
  mixer = new AnimationMixer(this.root);

  #animationsEnabled = false;
  #animationActions = new Map<string, AnimationAction>();

  builders = {
    texture: new TextureBuilder(this),
    material: new MaterialBuilder(this),
    primitive: new PrimitiveBuilder(this),
    mesh: new MeshBuilder(this),
    node: new NodeBuilder(this),
    skin: new SkinBuilder(this),
    animation: new AnimationBuilder(this),
  };

  constructor(toMainThread: (message: SceneMessage) => void) {
    super();

    this.#toMainThread = toMainThread;
  }

  get csm() {
    return this.#csm;
  }

  set csm(csm: CSM | null) {
    this.#csm?.dispose();
    this.#csm = csm;

    for (const [, material] of this.builders.material.listObjects()) {
      csm?.setupMaterial(material);
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
        this.createAccessor(data.json, data.id);
        break;
      }

      case "dispose_accessor": {
        const accessor = this.accessor.store.get(data);
        if (accessor) accessor.dispose();
        break;
      }

      case "create_texture": {
        this.builders.texture.add(data.json, data.id);
        break;
      }

      case "dispose_texture": {
        this.builders.texture.remove(data);
        break;
      }

      case "create_material": {
        this.builders.material.add(data.json, data.id);

        const object = this.builders.material.getObject(data.id);
        if (!object) throw new Error("Material not found");

        this.csm?.setupMaterial(object);
        break;
      }

      case "change_material": {
        this.builders.material.update(data.json, data.id);
        break;
      }

      case "dispose_material": {
        this.builders.material.remove(data);
        break;
      }

      case "create_primitive": {
        this.builders.primitive.add(data.json, data.id);
        break;
      }

      case "change_primitive": {
        this.builders.primitive.update(data.json, data.id);
        break;
      }

      case "dispose_primitive": {
        this.builders.primitive.remove(data);
        break;
      }

      case "create_mesh": {
        this.builders.mesh.add(data.json, data.id);
        break;
      }

      case "change_mesh": {
        this.builders.mesh.update(data.json, data.id);
        break;
      }

      case "dispose_mesh": {
        this.builders.mesh.remove(data);
        break;
      }

      case "create_node": {
        this.builders.node.add(data.json, data.id);
        break;
      }

      case "change_node": {
        this.builders.node.update(data.json, data.id);
        break;
      }

      case "dispose_node": {
        this.builders.node.remove(data);
        break;
      }

      case "create_skin": {
        this.builders.skin.add(data.json, data.id);
        break;
      }

      case "dispose_skin": {
        this.builders.skin.remove(data);
        break;
      }

      case "create_animation": {
        this.builders.animation.add(data.json, data.id);

        const object = this.builders.animation.getObject(data.id);
        if (!object) throw new Error("Animation not found");

        const action = this.mixer.clipAction(object);
        this.#animationActions.set(data.id, action);

        if (this.#animationsEnabled) action.play();
        break;
      }

      case "change_animation": {
        this.builders.animation.update(data.json, data.id);
        break;
      }

      case "dispose_animation": {
        this.builders.animation.remove(data);
        this.#animationActions.delete(data);
        break;
      }
    }
  }

  createAccessor(json: Partial<AccessorJSON>, accessorId?: string) {
    if (accessorId) {
      const accessor = this.accessor.store.get(accessorId);
      if (accessor) return { id: accessorId, accessor };
    }

    const { id, object: accessor } = this.accessor.create(json, accessorId);

    if (!accessorId) {
      const fullJSON = this.accessor.toJSON(accessor);
      this.#toMainThread({ subject: "create_accessor", data: { id, json: fullJSON } });
    }

    return { id, accessor };
  }

  getInstancedMeshNodeId(object: Object3D): string | null {
    // for (const [id, clonedMeshObject] of this.instancedMeshObjects.entries()) {
    //   let found = false;
    //   clonedMeshObject.traverse((child) => {
    //     if (child === object) found = true;
    //   });
    //   if (found) return id;
    // }

    return null;
  }

  getPrimitiveId(object: Object3D): string | null {
    for (const [id, primitiveObject] of this.builders.primitive.listObjects()) {
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
    for (const [id, meshObject] of this.builders.mesh.listObjects()) {
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
    for (const [id, nodeObject] of this.builders.node.listObjects()) {
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

  toggleAnimations(enabled: boolean) {
    this.#animationsEnabled = enabled;

    if (enabled) this.#animationActions.forEach((action) => action.play());
    else this.mixer.stopAllAction();
  }

  destroy() {
    this.root.removeFromParent();
    deepDispose(this.root);
  }
}

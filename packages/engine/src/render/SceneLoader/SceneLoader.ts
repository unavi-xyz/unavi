import type { GLTF } from "@gltf-transform/core";
import {
  AnimationClip,
  AnimationMixer,
  Box3,
  BufferAttribute,
  BufferGeometry,
  CylinderGeometry,
  DirectionalLight,
  Group,
  Mesh,
  MeshBasicMaterial,
  MeshStandardMaterial,
  Object3D,
  Vector3,
} from "three";

import { AccessorJSON, MeshJSON, NodeJSON } from "../../scene";
import { sortNodes } from "../../scene/utils/sortNodes";
import { PostMessage, Quad } from "../../types";
import { FromRenderMessage, RenderExport, ToRenderMessage } from "../types";
import { addAnimation } from "./animation/addAnimation";
import { addMaterial } from "./material/addMaterial";
import { removeMaterial } from "./material/removeMaterial";
import { updateMaterial } from "./material/updateMaterial";
import { addNode } from "./node/addNode";
import { createColliderVisual } from "./node/createColliderVisual";
import { removeNode } from "./node/removeNode";
import { updateNode } from "./node/updateNode";
import { SceneMap } from "./types";
import { getChildren } from "./utils/getChildren";
import { updateGlobalTransform } from "./utils/updateGlobalTransform";

/*
 * Turns the {@link RenderScene} into a Three.js scene.
 */
export class SceneLoader {
  root = new Group();
  contents = new Group();
  visuals = new Group();
  mixer = new AnimationMixer(this.root);

  #sun = new DirectionalLight(0xfff0db, 0.98);
  #spawn = new Mesh(
    new CylinderGeometry(0.5, 0.5, 1.6, 8),
    new MeshBasicMaterial({ wireframe: true })
  );

  #map: SceneMap = {
    accessors: new Map<string, AccessorJSON>(),
    animations: new Map<string, AnimationClip>(),
    attributes: new Map<string, BufferAttribute>(),
    colliders: new Map<string, Group>(),
    nodes: new Map<string, NodeJSON>(),
    meshes: new Map<string, MeshJSON>(),
    images: new Map<string, ImageBitmap>(),
    materials: new Map<string, MeshStandardMaterial>(),
    objects: new Map<string, Object3D>(),
  };

  #postMessage: PostMessage<FromRenderMessage>;

  constructor(postMessage: PostMessage<FromRenderMessage>) {
    this.#postMessage = postMessage;

    this.root.add(this.visuals);
    this.root.add(this.contents);
    this.#map.objects.set("root", this.contents);

    this.visuals.visible = false;
    this.visuals.add(this.#spawn);

    this.#sun.castShadow = true;
    this.#sun.position.set(10, 50, 30);
    this.root.add(this.#sun);

    this.#sun.shadow.mapSize.width = 4096;
    this.#sun.shadow.mapSize.height = 4096;
  }

  onmessage = (event: MessageEvent<ToRenderMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "show_visuals": {
        this.visuals.visible = data.visible;
        break;
      }

      case "add_node": {
        addNode(data.node, this.#map, this.visuals, this.#postMessage);
        this.#updateShadowMap();
        break;
      }

      case "remove_node": {
        removeNode(data.nodeId, this.#map);
        this.#updateShadowMap();
        break;
      }

      case "update_node": {
        updateNode(
          data.nodeId,
          data.data,
          this.#map,
          this.visuals,
          this.#postMessage
        );
        this.#updateShadowMap();
        break;
      }

      case "add_material": {
        addMaterial(data.material, this.#map);
        break;
      }

      case "remove_material": {
        removeMaterial(data.materialId, this.#map);
        break;
      }

      case "update_material": {
        updateMaterial(data.materialId, data.data, this.#map);
        break;
      }

      case "load_json": {
        // Set spawn
        if (data.scene.spawn) {
          this.#spawn.position.fromArray(data.scene.spawn);
          this.#spawn.position.y += 0.8;
        }

        // Add accessors
        if (data.scene.accessors)
          data.scene.accessors.forEach((a) => this.#map.accessors.set(a.id, a));

        // Add images
        if (data.scene.images)
          data.scene.images.forEach((i) =>
            this.#map.images.set(i.id, i.bitmap)
          );

        // Add materials
        if (data.scene.materials)
          data.scene.materials.forEach((m) => addMaterial(m, this.#map));

        // Add nodes
        if (data.scene.nodes) {
          const sortedNodes = sortNodes(data.scene);

          sortedNodes.forEach((e) =>
            addNode(e, this.#map, this.visuals, this.#postMessage)
          );

          // Hull collider visuals require all children to be added
          this.#map.nodes.forEach((e) => {
            if (e.collider?.type === "hull" || e.collider?.type === "mesh")
              createColliderVisual(
                e.id,
                this.#map,
                this.visuals,
                this.#postMessage
              );
          });
        }

        // Add animations
        if (data.scene.animations)
          data.scene.animations.forEach((a) => {
            addAnimation(a, this.#map, this.mixer);
          });

        this.#updateShadowMap();
        break;
      }

      case "prepare_export": {
        this.prepareExport();
        break;
      }
    }
  };

  prepareExport() {
    const exportData: RenderExport = [];

    function exportAttribute(
      nodeId: string,
      attributeName: string,
      threeName: string,
      mesh: Mesh<BufferGeometry, MeshStandardMaterial>
    ) {
      const attribute =
        attributeName === "indices"
          ? mesh.geometry.getIndex()
          : mesh.geometry.getAttribute(threeName);

      if (!attribute) return;

      const types = {
        1: "SCALAR",
        2: "VEC2",
        3: "VEC3",
        4: "VEC4",
        16: "MAT4",
      } as const;

      const itemSize: keyof typeof types = attribute.itemSize as any;
      const type: GLTF.AccessorType = types[itemSize];

      exportData.push({
        nodeId,
        attributeName,
        array: attribute.array as any,
        normalized: attribute.normalized,
        type,
      });
    }

    this.#map.nodes.forEach((node) => {
      const meshId = node.meshId;
      if (!meshId) return;

      const mesh = this.#map.meshes.get(meshId);
      if (!mesh) throw new Error(`Mesh ${meshId} not found`);

      switch (mesh.type) {
        case "Box":
        case "Sphere":
        case "Cylinder": {
          const object = this.findObject(node.id);
          if (!object) throw new Error("Object not found");
          if (!(object instanceof Mesh))
            throw new Error("Object is not a mesh");

          const mesh = object as Mesh<BufferGeometry, MeshStandardMaterial>;

          exportAttribute(node.id, "indices", "indices", mesh);
          exportAttribute(node.id, "POSITION", "position", mesh);
          exportAttribute(node.id, "NORMAL", "normal", mesh);
          exportAttribute(node.id, "TANGENT", "tangent", mesh);
          exportAttribute(node.id, "TEXCOORD_0", "uv", mesh);
          exportAttribute(node.id, "TEXCOORD_1", "tangent", mesh);
          exportAttribute(node.id, "COLOR_0", "color", mesh);
          break;
        }
      }
    });

    this.#postMessage({
      subject: "export",
      data: exportData,
    });
  }

  findId(target: Object3D): string | undefined {
    for (const [id, object] of this.#map.objects) {
      if (object === target) return id;
    }
    return undefined;
  }

  getNode(id: string) {
    return this.#map.nodes.get(id);
  }

  findObject(nodeId: string): Object3D | undefined {
    return this.#map.objects.get(nodeId);
  }

  saveTransform(nodeId: string) {
    const node = this.getNode(nodeId);
    if (!node) throw new Error(`Node not found: ${nodeId}`);

    const object = this.findObject(nodeId);
    if (!object) throw new Error("Object not found");

    const position = object.position.toArray();
    const scale = object.scale.toArray();

    const rotation: Quad = [
      object.quaternion.x,
      object.quaternion.y,
      object.quaternion.z,
      object.quaternion.w,
    ];

    this.#postMessage({
      subject: "set_transform",
      data: {
        nodeId,
        position,
        rotation,
        scale,
      },
    });

    // Update global transform
    updateGlobalTransform(node.id, this.#map, this.#postMessage);

    // Repeat for children
    const children = getChildren(node.id, this.#map);
    children.forEach((child) => this.saveTransform(child.id));

    this.#updateShadowMap();
  }

  #updateShadowMap() {
    const sceneBounds = new Box3();
    this.contents.traverse((object) => {
      if (object instanceof Mesh) {
        sceneBounds.expandByObject(object);
      }
    });

    const size = sceneBounds.getSize(new Vector3());

    const y = size.y + 20;
    this.#sun.position.set(0, y, 0);
    this.#sun.shadow.camera.far = y * 2;

    this.#sun.shadow.camera.left = -size.x / 2;
    this.#sun.shadow.camera.right = size.x / 2;
    this.#sun.shadow.camera.top = size.z / 2;
    this.#sun.shadow.camera.bottom = -size.z / 2;

    this.#sun.shadow.bias = -0.0005;

    this.#sun.shadow.camera.updateProjectionMatrix();
  }
}

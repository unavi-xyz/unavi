import type { GLTF } from "@gltf-transform/core";
import {
  AnimationClip,
  AnimationMixer,
  Box3,
  BufferAttribute,
  BufferGeometry,
  DirectionalLight,
  Group,
  Mesh,
  MeshStandardMaterial,
  Object3D,
  Vector3,
} from "three";

import { AccessorJSON, EntityJSON } from "../../scene";
import { sortEntities } from "../../scene/utils/sortEntities";
import { PostMessage, Quad } from "../../types";
import { FromRenderMessage, RenderExport, ToRenderMessage } from "../types";
import { addAnimation } from "./animation/addAnimation";
import { addEntity } from "./entity/addEntity";
import { removeEntity } from "./entity/removeEntity";
import { updateEntity } from "./entity/updateEntity";
import { addMaterial } from "./material/addMaterial";
import { removeMaterial } from "./material/removeMaterial";
import { updateMaterial } from "./material/updateMaterial";
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

  #map: SceneMap = {
    accessors: new Map<string, AccessorJSON>(),
    animations: new Map<string, AnimationClip>(),
    attributes: new Map<string, BufferAttribute>(),
    colliders: new Map<string, Mesh>(),
    entities: new Map<string, EntityJSON>(),
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

      case "add_entity": {
        addEntity(data.entity, this.#map, this.visuals, this.#postMessage);
        this.#updateShadowMap();
        break;
      }

      case "remove_entity": {
        removeEntity(data.entityId, this.#map);
        this.#updateShadowMap();
        break;
      }

      case "update_entity": {
        updateEntity(
          data.entityId,
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

        // Add entities
        if (data.scene.entities) {
          const sortedEntities = sortEntities(data.scene.entities);
          sortedEntities.forEach((e) =>
            addEntity(e, this.#map, this.visuals, this.#postMessage)
          );
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
      entityId: string,
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
        entityId,
        attributeName,
        array: attribute.array as any,
        normalized: attribute.normalized,
        type,
      });
    }

    this.#map.entities.forEach((entity) => {
      switch (entity.mesh?.type) {
        case "Box":
        case "Sphere":
        case "Cylinder": {
          const object = this.findObject(entity.id);
          if (!object) throw new Error("Object not found");
          if (!(object instanceof Mesh))
            throw new Error("Object is not a mesh");

          const mesh = object as Mesh<BufferGeometry, MeshStandardMaterial>;

          exportAttribute(entity.id, "indices", "indices", mesh);
          exportAttribute(entity.id, "POSITION", "position", mesh);
          exportAttribute(entity.id, "NORMAL", "normal", mesh);
          exportAttribute(entity.id, "TANGENT", "tangent", mesh);
          exportAttribute(entity.id, "TEXCOORD_0", "uv", mesh);
          exportAttribute(entity.id, "TEXCOORD_1", "tangent", mesh);
          exportAttribute(entity.id, "COLOR_0", "color", mesh);
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

  getEntity(id: string) {
    return this.#map.entities.get(id);
  }

  findObject(entityId: string): Object3D | undefined {
    return this.#map.objects.get(entityId);
  }

  saveTransform(entityId: string) {
    const entity = this.getEntity(entityId);
    if (!entity) throw new Error(`Entity not found: ${entityId}`);

    const object = this.findObject(entityId);
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
        entityId,
        position,
        rotation,
        scale,
      },
    });

    // Update global transform
    updateGlobalTransform(entity.id, this.#map, this.#postMessage);

    // Repeat for children
    const children = getChildren(entity.id, this.#map);
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

import {
  AnimationClip,
  AnimationMixer,
  BufferAttribute,
  Group,
  Mesh,
  MeshStandardMaterial,
  Object3D,
} from "three";

import { AccessorJSON, EntityJSON } from "../../scene";
import { PostMessage, Quad } from "../../types";
import { FromRenderMessage, ToRenderMessage } from "../types";
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
  }

  onmessage = (event: MessageEvent<ToRenderMessage>) => {
    const { subject, data } = event.data;
    switch (subject) {
      case "show_visuals":
        this.visuals.visible = data.visible;
        break;
      case "add_entity":
        this.#map.entities.set(data.entity.id, data.entity);
        updateEntity(
          data.entity.id,
          data.entity,
          this.#map,
          this.visuals,
          this.#postMessage
        );
        break;
      case "remove_entity":
        removeEntity(data.entityId, this.#map);
        break;
      case "update_entity":
        updateEntity(
          data.entityId,
          data.data,
          this.#map,
          this.visuals,
          this.#postMessage
        );
        break;
      case "add_material":
        addMaterial(data.material, this.#map);
        break;
      case "remove_material":
        removeMaterial(data.materialId, this.#map);
        break;
      case "update_material":
        updateMaterial(data.materialId, data.data, this.#map);
        break;
      case "load_json":
        if (data.scene.images)
          data.scene.images.forEach((i) =>
            this.#map.images.set(i.id, i.bitmap)
          );
        if (data.scene.accessors)
          data.scene.accessors.forEach((a) => this.#map.accessors.set(a.id, a));
        if (data.scene.materials)
          data.scene.materials.forEach((m) => addMaterial(m, this.#map));
        if (data.scene.entities) {
          data.scene.entities.forEach((e) => {
            this.#map.entities.set(e.id, e);
            updateEntity(e.id, e, this.#map, this.visuals, this.#postMessage);
          });
        }
        // if (data.scene.animations)
        //   data.scene.animations.forEach((a) =>  this.#map.animations.set(a.id, a));
        break;
    }
  };

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
  }
}

import {
  AnimationClip,
  AnimationMixer,
  BufferAttribute,
  Group,
  Mesh,
  MeshStandardMaterial,
  Object3D,
} from "three";

import { SceneMessage } from "../../scene";
import { PostMessage, Quad } from "../../types";
import { RenderScene } from "../RenderScene";
import { ToRenderMessage } from "../types";
import { createAnimation } from "./create/createAnimation";
import { createEntity } from "./create/createEntity";
import { createMaterial } from "./create/createMaterial";
import { SceneMap } from "./types";

/*
 * Turns the {@link RenderScene} into a Three.js scene.
 */
export class SceneLoader {
  root = new Group();
  contents = new Group();
  visuals = new Group();
  mixer = new AnimationMixer(this.root);

  #scene: RenderScene;

  #map: SceneMap = {
    objects: new Map<string, Object3D>(),
    materials: new Map<string, MeshStandardMaterial>(),
    attributes: new Map<string, BufferAttribute>(),
    animations: new Map<string, AnimationClip>(),
    colliders: new Map<string, Mesh>(),
  };

  constructor(postMessage: PostMessage) {
    this.#scene = new RenderScene(postMessage);

    this.root.add(this.visuals);
    this.root.add(this.contents);
    this.#map.objects.set("root", this.contents);

    // Add new entities
    this.#scene.entities$.subscribe({
      next: (entities) => {
        Object.values(entities).forEach((entity) => {
          if (!this.#map.objects.has(entity.id)) {
            createEntity(entity, this.#map, this.#scene, this.visuals);
          }
        });
      },
    });

    // Add new materials
    this.#scene.materials$.subscribe({
      next: (materials) => {
        Object.values(materials).forEach((material) => {
          if (!this.#map.materials.has(material.id)) {
            createMaterial(material, this.#map, this.#scene);
          }
        });
      },
    });

    // Add new animations
    this.#scene.animations$.subscribe({
      next: (animations) => {
        Object.values(animations).forEach((animation) => {
          if (!this.#map.animations.has(animation.id)) {
            createAnimation(animation, this.#map, this.#scene, this.mixer);
          }
        });
      },
    });
  }

  onmessage = (event: MessageEvent<ToRenderMessage>) => {
    this.#scene.onmessage(event as MessageEvent<SceneMessage>);

    const { subject, data } = event.data;
    switch (subject) {
      case "show_visuals":
        this.visuals.visible = data.visible;
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
    return this.#scene.entities[id];
  }

  findObject(entityId: string): Object3D | undefined {
    return this.#map.objects.get(entityId);
  }

  saveTransform(entityId: string) {
    const entity = this.#scene.entities[entityId];
    if (!entity) throw new Error(`Entity not found: ${entityId}`);

    const object = this.findObject(entityId);
    if (!object) throw new Error("Object not found");

    const position = object.position.toArray();
    const quaternion = object.quaternion.toArray();
    const scale = object.scale.toArray();

    const rotation: Quad = [
      quaternion[0],
      quaternion[1],
      quaternion[2],
      quaternion[3],
    ];

    this.#scene.updateEntity(entityId, {
      position,
      rotation,
      scale,
    });

    // Repeat for children
    entity.childrenIds.forEach((childId) => this.saveTransform(childId));
  }

  destroy() {
    this.#scene.destroy();
  }
}

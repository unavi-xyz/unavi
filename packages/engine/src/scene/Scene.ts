import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

import { PostMessage } from "../types";
import { Entity } from "./Entity";
import { Material } from "./Material";
import { EntityJSON, SceneJSON, SceneMessage } from "./types";

/*
 * Scene stores the state of the scene.
 * A copy of this class is created on the main thread, and in worker threads.
 * State is synced between the threads using postMessage.
 * State is stored using RxJS, allowing for subscriptions to state changes.
 */
export class Scene {
  #threads: PostMessage<SceneMessage>[] = [];

  #id = nanoid();

  entities$ = new BehaviorSubject<{ [id: string]: Entity }>({});
  materials$ = new BehaviorSubject<{ [id: string]: Material }>({});

  get entities() {
    return this.entities$.value;
  }

  set entities(entities: { [id: string]: Entity }) {
    this.entities$.next(entities);
  }

  get materials() {
    return this.materials$.value;
  }

  set materials(materials: { [id: string]: Material }) {
    this.materials$.next(materials);
  }

  constructor() {
    const root = new Entity({ id: "root" });
    this.#addEntity(root);
  }

  addThread(postMessage: PostMessage<SceneMessage>) {
    this.#threads.push(postMessage);

    // Send initial state
    const scene = this.toJSON();
    postMessage({
      subject: "load_json",
      data: { scene },
    });
  }

  removeThread(postMessage: PostMessage) {
    this.#threads = this.#threads.filter((t) => t !== postMessage);
  }

  #broadcast(message: SceneMessage) {
    this.#threads.forEach((postMessage) => postMessage(message));
  }

  onmessage = (event: MessageEvent<SceneMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "load_json":
        this.loadJSON(data.scene);
        break;
      case "add_entity":
        this.#addEntity(Entity.fromJSON(data.entity));
        break;
      case "remove_entity":
        this.#removeEntity(data.entityId);
        break;
      case "update_entity":
        this.#updateEntity(data.entityId, data.data);
        break;
      case "add_material":
        this.#addMaterial(Material.fromJSON(data.material));
        break;
      case "remove_material":
        this.#removeMaterial(data.materialId);
        break;
      case "update_material":
        this.#updateMaterial(data.materialId, data.data);
        break;
    }
  };

  addEntity(entity: Entity) {
    this.#addEntity(entity);
    this.#broadcast({
      subject: "add_entity",
      data: { entity: entity.toJSON() },
    });
  }

  #addEntity(entity: Entity) {
    const previous = this.entities[entity.id];
    if (previous) this.removeEntity(previous.id);

    // Set scene
    entity.scene = this;

    // Add to parent
    const parent = entity.parent;
    if (parent) {
      parent.childrenIds$.next([...parent.childrenIds$.value, entity.id]);
    }

    // Save to entities
    this.entities = {
      ...this.entities,
      [entity.id]: entity,
    };
  }

  removeEntity(entityId: string) {
    this.#removeEntity(entityId);
    this.#broadcast({ subject: "remove_entity", data: { entityId } });
  }

  #removeEntity(entityId: string) {
    const entity = this.entities[entityId];
    if (!entity) throw new Error(`Entity ${entityId} not found`);

    // Remove from parent
    if (entity.parent) {
      entity.parent.childrenIds$.next(
        entity.parent.childrenIds$.value.filter((id) => id !== entityId)
      );
    }

    // Destroy entity
    entity.destroy();

    // Remove from entities
    this.entities = Object.fromEntries(
      Object.entries(this.entities).filter(([id]) => id !== entityId)
    );
  }

  updateEntity(entityId: string, data: Partial<EntityJSON>) {
    this.#updateEntity(entityId, data);
    this.#broadcast({ subject: "update_entity", data: { entityId, data } });
  }

  #updateEntity(entityId: string, data: Partial<EntityJSON>) {
    const entity = this.entities[entityId];
    if (!entity) throw new Error(`Entity ${entityId} not found`);

    entity.applyJSON(data);
  }

  addMaterial(material: Material) {
    this.#addMaterial(material);
    this.#broadcast({
      subject: "add_material",
      data: { material: material.toJSON() },
    });
  }

  #addMaterial(material: Material) {
    const previous = this.materials[material.id];
    if (previous) this.removeMaterial(previous.id);

    // Save to materials
    this.materials = {
      ...this.materials,
      [material.id]: material,
    };
  }

  removeMaterial(materialId: string) {
    this.#removeMaterial(materialId);
    this.#broadcast({ subject: "remove_material", data: { materialId } });
  }

  #removeMaterial(materialId: string) {
    const material = this.materials[materialId];
    if (!material) throw new Error(`Material ${materialId} not found`);

    // Remove from all entities
    this.entities = Object.fromEntries(
      Object.entries(this.entities).map(([id, entity]) => {
        if (entity.materialId === materialId) entity.materialId = null;
        return [id, entity];
      })
    );

    // Destroy material
    material.destroy();

    // Remove from materials
    this.materials = Object.fromEntries(
      Object.entries(this.materials).filter(([id]) => id !== materialId)
    );
  }

  updateMaterial(materialId: string, data: Partial<Material>) {
    this.#updateMaterial(materialId, data);
    this.#broadcast({ subject: "update_material", data: { materialId, data } });
  }

  #updateMaterial(materialId: string, data: Partial<Material>) {
    const material = this.materials[materialId];
    if (!material) throw new Error(`Material ${materialId} not found`);

    material.applyJSON(data);
  }

  toJSON(): SceneJSON {
    return {
      entities: Object.values(this.entities).map((e) => e.toJSON()),
      materials: Object.values(this.materials).map((m) => m.toJSON()),
    };
  }

  loadJSON(json: SceneJSON) {
    json.materials.forEach((material) =>
      this.addMaterial(Material.fromJSON(material))
    );
    json.entities.forEach((entity) => this.addEntity(Entity.fromJSON(entity)));
  }

  destroy() {
    Object.values(this.entities).forEach((entity) => entity.destroy());
    Object.values(this.materials).forEach((material) => material.destroy());

    this.entities$.complete();
    this.materials$.complete();
  }
}

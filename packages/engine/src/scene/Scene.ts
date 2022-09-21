import { BehaviorSubject } from "rxjs";

import { PostMessage } from "../types";
import { Entity } from "./Entity";
import { Material } from "./Material";
import { EntityJSON, SceneJSON, SceneMessage } from "./types";

/*
 * This class holds the scene state.
 * A copy of this class is created on the main thread, and in worker threads.
 * State is automatically synced between the threads.
 */
export class Scene {
  #threads: PostMessage<SceneMessage>[] = [];

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

  addThread(postMessage: PostMessage) {
    this.#threads.push(postMessage);
  }
  removeThread(postMessage: PostMessage) {
    this.#threads = this.#threads.filter((t) => t !== postMessage);
  }

  #broadcast(message: SceneMessage) {
    this.#threads.forEach((postMessage) => postMessage(message));
  }

  onmessage = ({ subject, data }: SceneMessage) => {
    switch (subject) {
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
    }
  };

  // Entities
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
    // Remove from parent
    const entity = this.entities[entityId];
    if (entity) {
      const parent = entity.parent;
      if (parent) {
        parent.childrenIds$.next(
          parent.childrenIds$.value.filter((id) => id !== entityId)
        );
      }
    }

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
    entity.applyJSON(data);
  }

  // Materials
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
    // Remove from all entities
    this.entities = Object.fromEntries(
      Object.entries(this.entities).map(([id, entity]) => {
        if (entity.materialId === materialId) entity.materialId = null;
        return [id, entity];
      })
    );

    // Remove material
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
    material.applyJSON(data);
  }

  // JSON
  toJSON(): SceneJSON {
    return {
      entities: Object.values(this.entities).map((e) => e.toJSON()),
      materials: Object.values(this.materials).map((m) => m.toJSON()),
    };
  }

  loadJSON(json: SceneJSON) {
    json.entities.forEach((entity) => this.addEntity(Entity.fromJSON(entity)));
    json.materials.forEach((material) =>
      this.addMaterial(Material.fromJSON(material))
    );
  }
}

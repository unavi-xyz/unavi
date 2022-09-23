import { BehaviorSubject } from "rxjs";

import { GameThread } from "../game/GameThread";
import { ToGameMessage } from "../game/types";
import { RenderThread } from "../render/RenderThread";
import { ToRenderMessage } from "../render/types";
import { Entity } from "../scene/Entity";
import { Material } from "../scene/Material";
import { EntityJSON, SceneJSON, SceneMessage } from "../scene/types";
import { PostMessage, Triplet } from "../types";

/*
 * MainScene stores the complete state of the scene.
 * Scene state is synced between the MainScene, {@link RenderScene}, and {@link GameScene}.
 * State is stored using RxJS, allowing for subscriptions to state changes.
 */
export class MainScene {
  #toRenderThread: PostMessage<ToRenderMessage>;
  #toGameThread: PostMessage<ToGameMessage>;

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

  constructor({
    renderThread,
    gameThread,
  }: {
    renderThread: RenderThread;
    gameThread: GameThread;
  }) {
    this.#toRenderThread = renderThread.postMessage.bind(renderThread);
    this.#toGameThread = gameThread.postMessage.bind(gameThread);

    const root = new Entity({ id: "root" });
    this.#addEntity(root);
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
      case "update_global_transform":
        this.updateGlobalTransform(
          data.entityId,
          data.position,
          data.quaternion
        );
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

    const message: SceneMessage = {
      subject: "add_entity",
      data: { entity: entity.toJSON() },
    };

    this.#toRenderThread(message);
    this.#toGameThread(message);
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

    const message: SceneMessage = {
      subject: "remove_entity",
      data: { entityId },
    };

    this.#toRenderThread(message);
    this.#toGameThread(message);
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

    const message: SceneMessage = {
      subject: "update_entity",
      data: { entityId, data },
    };

    this.#toRenderThread(message);
    this.#toGameThread(message);
  }

  #updateEntity(entityId: string, data: Partial<EntityJSON>) {
    const entity = this.entities[entityId];
    if (!entity) throw new Error(`Entity ${entityId} not found`);

    entity.applyJSON(data);
  }

  updateGlobalTransform(
    entityId: string,
    position: Triplet,
    quaternion: [number, number, number, number]
  ) {
    const entity = this.entities[entityId];
    entity.globalPosition = position;
    entity.globalQuaternion = quaternion;

    const message: SceneMessage = {
      subject: "update_global_transform",
      data: {
        entityId,
        position,
        quaternion,
      },
    };

    this.#toGameThread(message);
  }

  addMaterial(material: Material) {
    this.#addMaterial(material);
    this.#toRenderThread({
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
    this.#toRenderThread({ subject: "remove_material", data: { materialId } });
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
    this.#toRenderThread({
      subject: "update_material",
      data: { materialId, data },
    });
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

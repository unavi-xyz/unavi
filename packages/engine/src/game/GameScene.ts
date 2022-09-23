import { BehaviorSubject } from "rxjs";

import { Entity, EntityJSON, SceneMessage } from "../scene";
import { PostMessage, Triplet } from "../types";
import { FromGameMessage } from "./types";

/*
 * GameScene stores the scene state needed for the {@link GameWorker}.
 * State is synced with the {@link MainScene}.
 */
export class GameScene {
  #toMainThread: PostMessage<FromGameMessage>;

  constructor(postMessage: PostMessage<FromGameMessage>) {
    this.#toMainThread = postMessage;

    const root = new Entity({ id: "root" });
    this.#addEntity(root);
  }

  entities$ = new BehaviorSubject<{ [id: string]: Entity }>({});

  get entities() {
    return this.entities$.value;
  }

  set entities(entities: { [id: string]: Entity }) {
    this.entities$.next(entities);
  }

  onmessage = (event: MessageEvent<SceneMessage>) => {
    const { subject, data } = event.data;

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
      case "update_global_transform":
        this.#updateGlobalTransform(
          data.entityId,
          data.position,
          data.quaternion
        );
        break;
    }
  };

  #addEntity(entity: Entity) {
    const previous = this.entities[entity.id];
    if (previous) this.#removeEntity(previous.id);

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

  #updateEntity(entityId: string, data: Partial<EntityJSON>) {
    const entity = this.entities[entityId];
    if (!entity) throw new Error(`Entity ${entityId} not found`);

    entity.applyJSON(data);
  }

  #updateGlobalTransform(
    entityId: string,
    position: Triplet,
    quaternion: [number, number, number, number]
  ) {
    const entity = this.entities[entityId];
    entity.globalPosition = position;
    entity.globalQuaternion = quaternion;
  }

  destroy() {
    Object.values(this.entities).forEach((entity) => entity.destroy());
    this.entities$.complete();
  }
}

import { BehaviorSubject } from "rxjs";

import { Quad, Triplet } from "../types";
import { Accessor } from "./Accessor";
import { Animation } from "./Animation";
import { Entity } from "./Entity";
import { Image } from "./Image";
import { Material } from "./Material";
import { EntityJSON, MaterialJSON, SceneJSON } from "./types";
import { sortEntities } from "./utils/sortEntities";

/*
 * Stores the state of the scene.
 * State is stored using RxJS, allowing for subscriptions to state changes.
 */
export class Scene {
  entities$ = new BehaviorSubject<{ [id: string]: Entity }>({});
  materials$ = new BehaviorSubject<{ [id: string]: Material }>({});
  accessors$ = new BehaviorSubject<{ [id: string]: Accessor }>({});
  images$ = new BehaviorSubject<{ [id: string]: Image }>({});
  animations$ = new BehaviorSubject<{ [id: string]: Animation }>({});

  constructor() {
    const root = new Entity({ id: "root" });
    this.addEntity(root);
  }

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

  get accessors() {
    return this.accessors$.value;
  }

  set accessors(accessors: { [id: string]: Accessor }) {
    this.accessors$.next(accessors);
  }

  get images() {
    return this.images$.value;
  }

  set images(images: { [id: string]: Image }) {
    this.images$.next(images);
  }

  get animations() {
    return this.animations$.value;
  }

  set animations(animations: { [id: string]: Animation }) {
    this.animations$.next(animations);
  }

  addAccessor(accessor: Accessor) {
    const previous = this.accessors[accessor.id];
    if (previous) this.removeAccessor(previous.id);

    // Save to accessors
    this.accessors = {
      ...this.accessors,
      [accessor.id]: accessor,
    };
  }

  removeAccessor(accessorId: string) {
    const accessors = { ...this.accessors };
    delete accessors[accessorId];
    this.accessors = accessors;
  }

  addEntity(entity: Entity) {
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
    const entity = this.entities[entityId];
    if (!entity) throw new Error(`Entity ${entityId} not found`);

    entity.applyJSON(data);
  }

  updateGlobalTransform(entityId: string, position: Triplet, quaternion: Quad) {
    const entity = this.entities[entityId];
    entity.globalPosition = position;
    entity.globalQuaternion = quaternion;
  }

  addMaterial(material: Material) {
    const previous = this.materials[material.id];
    if (previous) this.removeMaterial(previous.id);

    // Save to materials
    this.materials = {
      ...this.materials,
      [material.id]: material,
    };
  }

  removeMaterial(materialId: string) {
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

  updateMaterial(materialId: string, data: Partial<MaterialJSON>) {
    const material = this.materials[materialId];
    if (!material) throw new Error(`Material ${materialId} not found`);

    material.applyJSON(data);
  }

  addImage(image: Image) {
    const previous = this.images[image.id];
    if (previous) this.removeImage(previous.id);

    // Save to images
    this.images = {
      ...this.images,
      [image.id]: image,
    };
  }

  removeImage(imageId: string) {
    const image = this.images[imageId];
    if (!image) throw new Error(`Image ${imageId} not found`);

    // Remove from images
    this.images = Object.fromEntries(
      Object.entries(this.images).filter(([id]) => id !== imageId)
    );
  }

  addAnimation(animation: Animation) {
    const previous = this.animations[animation.id];
    if (previous) this.removeAnimation(previous.id);

    // Save to animations
    this.animations = {
      ...this.animations,
      [animation.id]: animation,
    };
  }

  removeAnimation(animationId: string) {
    const animation = this.animations[animationId];
    if (!animation) throw new Error(`Animation ${animationId} not found`);

    // Remove from animations
    this.animations = Object.fromEntries(
      Object.entries(this.animations).filter(([id]) => id !== animationId)
    );
  }

  toJSON(includeInternal = false): SceneJSON {
    return {
      entities: Object.values(this.entities)
        .filter((e) => (e.isInternal ? includeInternal : true))
        .map((e) => e.toJSON()),
      materials: Object.values(this.materials)
        .filter((m) => (m.isInternal ? includeInternal : true))
        .map((m) => m.toJSON()),
      accessors: Object.values(this.accessors)
        .filter((a) => (a.isInternal ? includeInternal : true))
        .map((a) => a.toJSON()),
      images: Object.values(this.images)
        .filter((i) => (i.isInternal ? includeInternal : true))
        .map((i) => i.toJSON()),
      animations: Object.values(this.animations)
        .filter((a) => (a.isInternal ? includeInternal : true))
        .map((a) => a.toJSON()),
    };
  }

  loadJSON(json: Partial<SceneJSON>) {
    // Add accessors
    if (json.accessors) {
      json.accessors.forEach((accessor) =>
        this.addAccessor(Accessor.fromJSON(accessor))
      );
    }

    // Add images
    if (json.images) {
      json.images.forEach((image) => this.addImage(Image.fromJSON(image)));
    }

    // Add materials
    if (json.materials) {
      json.materials.forEach((material) =>
        this.addMaterial(Material.fromJSON(material))
      );
    }

    // Sort entities
    if (json.entities) {
      const sortedEntities = sortEntities(json.entities);

      // Add entities
      sortedEntities.forEach((entity) =>
        this.addEntity(Entity.fromJSON(entity))
      );
    }

    // Add animations
    if (json.animations) {
      json.animations.forEach((animation) =>
        this.addAnimation(Animation.fromJSON(animation))
      );
    }
  }

  destroy() {
    Object.values(this.entities).forEach((entity) => entity.destroy());
    Object.values(this.materials).forEach((material) => material.destroy());
    Object.values(this.accessors).forEach((accessor) => accessor.destroy());
    Object.values(this.images).forEach((image) => image.destroy());
    Object.values(this.animations).forEach((animation) => animation.destroy());

    this.entities$.complete();
    this.materials$.complete();
    this.accessors$.complete();
    this.images$.complete();
    this.animations$.complete();
  }
}

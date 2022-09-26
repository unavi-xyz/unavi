import { BehaviorSubject } from "rxjs";

import { Quad, Triplet } from "../types";
import { Accessor } from "./Accessor";
import { Entity } from "./Entity";
import { Image } from "./Image";
import { Material } from "./Material";
import { Texture } from "./Texture";
import { EntityJSON, SceneJSON } from "./types";

/*
 * Stores the state of the scene.
 * State is stored using RxJS, allowing for subscriptions to state changes.
 */
export class Scene {
  entities$ = new BehaviorSubject<{ [id: string]: Entity }>({});
  materials$ = new BehaviorSubject<{ [id: string]: Material }>({});
  textures$ = new BehaviorSubject<{ [id: string]: Texture }>({});
  accessors$ = new BehaviorSubject<{ [id: string]: Accessor }>({});
  images$ = new BehaviorSubject<{ [id: string]: Image }>({});

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

  get textures() {
    return this.textures$.value;
  }

  set textures(textures: { [id: string]: Texture }) {
    this.textures$.next(textures);
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

  updateMaterial(materialId: string, data: Partial<Material>) {
    const material = this.materials[materialId];
    if (!material) throw new Error(`Material ${materialId} not found`);

    material.applyJSON(data);
  }

  addTexture(texture: Texture) {
    const previous = this.textures[texture.id];
    if (previous) this.removeTexture(previous.id);

    // Save to textures
    this.textures = {
      ...this.textures,
      [texture.id]: texture,
    };
  }

  removeTexture(textureId: string) {
    const texture = this.textures[textureId];
    if (!texture) throw new Error(`Texture ${textureId} not found`);

    // Remove from all materials
    this.materials = Object.fromEntries(
      Object.entries(this.materials).map(([id, material]) => {
        // if (material.textureId === textureId) material.textureId = null;
        return [id, material];
      })
    );

    // Destroy texture
    texture.destroy();

    // Remove from textures
    this.textures = Object.fromEntries(
      Object.entries(this.textures).filter(([id]) => id !== textureId)
    );
  }

  updateTexture(textureId: string, data: Partial<Texture>) {
    const texture = this.textures[textureId];
    if (!texture) throw new Error(`Texture ${textureId} not found`);

    texture.applyJSON(data);
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

    // Remove from all textures
    this.textures = Object.fromEntries(
      Object.entries(this.textures).map(([id, texture]) => {
        if (texture.sourceId === imageId) texture.sourceId = null;
        return [id, texture];
      })
    );

    // Remove from images
    this.images = Object.fromEntries(
      Object.entries(this.images).filter(([id]) => id !== imageId)
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
      textures: Object.values(this.textures)
        .filter((t) => (t.isInternal ? includeInternal : true))
        .map((t) => t.toJSON()),
      accessors: Object.values(this.accessors)
        .filter((a) => (a.isInternal ? includeInternal : true))
        .map((a) => a.toJSON()),
      images: Object.values(this.images)
        .filter((i) => (i.isInternal ? includeInternal : true))
        .map((i) => i.toJSON()),
    };
  }

  loadJSON(json: SceneJSON) {
    json.accessors.forEach((accessor) =>
      this.addAccessor(Accessor.fromJSON(accessor))
    );
    json.images.forEach((image) => this.addImage(Image.fromJSON(image)));
    json.textures.forEach((texture) =>
      this.addTexture(Texture.fromJSON(texture))
    );
    json.materials.forEach((material) =>
      this.addMaterial(Material.fromJSON(material))
    );

    // Sort entities by parent, so that parents are created before children
    const sortedEntities = json.entities.sort((a, b) => {
      if (a.parentId === b.id) return 1;
      if (b.parentId === a.id) return -1;
      return 0;
    });
    sortedEntities.forEach((entity) => this.addEntity(Entity.fromJSON(entity)));
  }

  destroy() {
    Object.values(this.entities).forEach((entity) => entity.destroy());
    Object.values(this.materials).forEach((material) => material.destroy());
    Object.values(this.textures).forEach((texture) => texture.destroy());
    Object.values(this.accessors).forEach((accessor) => accessor.destroy());
    Object.values(this.images).forEach((image) => image.destroy());

    this.entities$.complete();
    this.materials$.complete();
    this.textures$.complete();
    this.accessors$.complete();
    this.images$.complete();
  }
}

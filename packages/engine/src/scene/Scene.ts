import { BehaviorSubject } from "rxjs";

import { Triplet } from "../types";
import { Accessor } from "./Accessor";
import { Animation } from "./Animation";
import { Entity } from "./Entity";
import { Image } from "./Image";
import { Material } from "./Material";
import { EntityJSON, MaterialJSON, SceneJSON } from "./types";
import { sortEntities } from "./utils/sortEntities";

/*
 * Stores the scene in a custom internal format.
 * State is stored using RxJS, allowing for subscriptions to state changes.
 * This is especially useful for the editor's React UI.
 */
export class Scene {
  entities$ = new BehaviorSubject<{ [id: string]: Entity }>({
    root: new Entity({ id: "root" }),
  });
  materials$ = new BehaviorSubject<{ [id: string]: Material }>({});
  accessors$ = new BehaviorSubject<{ [id: string]: Accessor }>({});
  images$ = new BehaviorSubject<{ [id: string]: Image }>({});
  animations$ = new BehaviorSubject<{ [id: string]: Animation }>({});

  spawn$ = new BehaviorSubject<Triplet | null>(null);

  get spawn() {
    return this.spawn$.value;
  }

  set spawn(spawn: Triplet | null) {
    this.spawn$.next(spawn);
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
    if (entity.id === "root") return;

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
    if (entityId === "root") return;

    const entity = this.entities[entityId];
    if (!entity) throw new Error(`Entity ${entityId} not found`);

    // Repeat for children
    entity.childrenIds.forEach((childId) => this.removeEntity(childId));

    // Remove from parent
    if (entity.parent) entity.parentId = "";

    // Remove from entities
    this.entities = Object.fromEntries(
      Object.entries(this.entities).filter(([id]) => id !== entityId)
    );

    // Remove mesh accessors
    if (entity.mesh && entity.mesh.type === "Primitive") {
      if (entity.mesh.indicesId) this.removeAccessor(entity.mesh.indicesId);
      if (entity.mesh.POSITION) this.removeAccessor(entity.mesh.POSITION);
      if (entity.mesh.NORMAL) this.removeAccessor(entity.mesh.NORMAL);
      if (entity.mesh.TEXCOORD_0) this.removeAccessor(entity.mesh.TEXCOORD_0);
      if (entity.mesh.TANGENT) this.removeAccessor(entity.mesh.TANGENT);
      if (entity.mesh.WEIGHTS_0) this.removeAccessor(entity.mesh.WEIGHTS_0);
      if (entity.mesh.JOINTS_0) this.removeAccessor(entity.mesh.JOINTS_0);

      entity.mesh.morphPositionIds.forEach((id) => this.removeAccessor(id));
      entity.mesh.morphNormalIds.forEach((id) => this.removeAccessor(id));
      entity.mesh.morphTangentIds.forEach((id) => this.removeAccessor(id));
    }

    // Remove material
    if (entity.materialId) {
      const material = this.materials[entity.materialId];
      if (!material) throw new Error(`Material ${entity.materialId} not found`);

      // Only remove internal materials
      if (material.isInternal) {
        // Only remove material if it's not used by any other entity
        const otherEntity = Object.values(this.entities).find(
          (e) => e.materialId === entity.materialId
        );

        if (!otherEntity) this.removeMaterial(entity.materialId);
      }
    }

    // Remove animations
    Object.values(this.animations).forEach((animation) => {
      // Only remove internal animations
      if (!animation.isInternal) return;

      // Remove animation if it doesn't have any other entity using it
      const targetIds = animation.channels.map((channel) => channel.targetId);
      const isUsed = targetIds.some((targetId) => {
        const targetEntity = this.entities[targetId];
        if (!targetEntity) return false;
        return true;
      });

      if (!isUsed) this.removeAnimation(animation.id);
    });

    // Destroy entity
    entity.destroy();
  }

  updateEntity(entityId: string, data: Partial<EntityJSON>) {
    if (entityId === "root") return;

    const entity = this.entities[entityId];
    if (!entity) throw new Error(`Entity ${entityId} not found`);

    entity.applyJSON(data);
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

    // Remove images
    [
      material.colorTexture,
      material.normalTexture,
      material.occlusionTexture,
      material.emissiveTexture,
      material.metallicRoughnessTexture,
    ].forEach((texture) => {
      if (!texture) return;
      const imageId = texture.imageId;
      if (!imageId) return;
      const image = this.images[imageId];
      if (!image) throw new Error(`Image ${imageId} not found`);

      // Only remove internal images
      if (!image.isInternal) return;

      // Only remove image if it's not used by any other material
      const otherMaterial = Object.values(this.materials).find((m) =>
        [
          m.colorTexture,
          m.normalTexture,
          m.occlusionTexture,
          m.emissiveTexture,
          m.metallicRoughnessTexture,
        ].some((t) => t?.imageId === imageId)
      );

      if (!otherMaterial) this.removeImage(imageId);
    });

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

    // Remove sampler accessors
    animation.channels.forEach((channel) => {
      this.removeAccessor(channel.sampler.inputId);
      this.removeAccessor(channel.sampler.outputId);
    });
  }

  toJSON(includeInternal = false): SceneJSON {
    return {
      spawn: this.spawn,

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
    if (json.spawn) this.spawn = json.spawn;

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
      sortedEntities.forEach((entity) => {
        if (entity.id === "root") return;
        this.addEntity(Entity.fromJSON(entity));
      });
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

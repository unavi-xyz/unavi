import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

import { Triplet } from "../types";
import { BoxMesh } from "./BoxMesh";
import { CylinderMesh } from "./CylinderMesh";
import { Scene } from "./Scene";
import { SphereMesh } from "./SphereMesh";
import { EntityJSON, Mesh, MeshJSON } from "./types";

/*
 * This class represents an entity in the scene.
 */
export class Entity {
  scene$ = new BehaviorSubject<Scene | null>(null);

  get scene() {
    return this.scene$.value;
  }

  set scene(scene: Scene | null) {
    this.scene$.next(scene);
  }

  id: string;
  name$ = new BehaviorSubject("Entity");

  parentId$ = new BehaviorSubject<string>("root");
  childrenIds$ = new BehaviorSubject<string[]>([]);

  position$ = new BehaviorSubject<Triplet>([0, 0, 0]);
  rotation$ = new BehaviorSubject<Triplet>([0, 0, 0]);
  scale$ = new BehaviorSubject<Triplet>([1, 1, 1]);

  mesh$ = new BehaviorSubject<Mesh | null>(null);
  materialId$ = new BehaviorSubject<string | null>(null);

  constructor({ id }: { id?: string } = {}) {
    this.id = id ?? nanoid();
  }

  get name() {
    return this.name$.value;
  }

  set name(name: string) {
    this.name$.next(name);
  }

  get parent() {
    return this.scene?.entities$.value[this.parentId$.value];
  }

  get parentId() {
    return this.parentId$.value;
  }

  set parentId(parentId: string) {
    // Remove from previous parent
    if (this.parent) {
      this.parent.childrenIds$.next(
        this.parent.childrenIds.filter((id) => id !== this.id)
      );
    }

    // Set new parent
    this.parentId$.next(parentId);

    // Add to new parent
    if (this.parent) {
      this.parent.childrenIds$.next([...this.parent.childrenIds, this.id]);
    }
  }

  get children() {
    return this.childrenIds$.value.map((id) => this.scene?.entities$.value[id]);
  }

  get childrenIds() {
    return this.childrenIds$.value;
  }

  get position() {
    return this.position$.value;
  }

  set position(position: Triplet) {
    this.position$.next(position);
  }

  get rotation() {
    return this.rotation$.value;
  }

  set rotation(rotation: Triplet) {
    this.rotation$.next(rotation);
  }

  get scale() {
    return this.scale$.value;
  }

  set scale(scale: Triplet) {
    this.scale$.next(scale);
  }

  get mesh() {
    return this.mesh$.value;
  }

  set mesh(mesh: Mesh | null) {
    this.mesh$.next(mesh);
  }

  get materialId() {
    return this.materialId$.value;
  }

  set materialId(materialId: string | null) {
    this.materialId$.next(materialId);
  }

  get material() {
    return this.materialId
      ? this.scene?.materials$.value[this.materialId]
      : null;
  }

  destroy() {
    this.name$.complete();
    this.parentId$.complete();
    this.childrenIds$.complete();
    this.position$.complete();
    this.rotation$.complete();
    this.scale$.complete();
    this.mesh$.complete();
    this.materialId$.complete();
  }

  toJSON(): EntityJSON {
    const mesh = this.mesh ? this.mesh.toJSON() : null;

    return {
      id: this.id,
      parentId: this.parentId,
      name: this.name,
      position: this.position,
      rotation: this.rotation,
      scale: this.scale,
      mesh,
      materialId: this.materialId,
    };
  }

  applyJSON(json: Partial<EntityJSON>) {
    if (json.name !== undefined) this.name = json.name;
    if (json.parentId !== undefined) this.parentId = json.parentId;

    if (json.position !== undefined) this.position = json.position;
    if (json.rotation !== undefined) this.rotation = json.rotation;
    if (json.scale !== undefined) this.scale = json.scale;

    if (json.mesh !== undefined) this.mesh = createMesh(json.mesh);
    if (json.materialId !== undefined) this.materialId = json.materialId;
  }

  static fromJSON(json: EntityJSON) {
    const entity = new Entity({ id: json.id });
    entity.applyJSON(json);
    return entity;
  }
}

function createMesh(json: MeshJSON | null) {
  if (!json) return null;

  switch (json.type) {
    case "Box":
      return BoxMesh.fromJSON(json);
    case "Sphere":
      return SphereMesh.fromJSON(json);
    case "Cylinder":
      return CylinderMesh.fromJSON(json);
    default:
      throw new Error("Unknown mesh type");
  }
}

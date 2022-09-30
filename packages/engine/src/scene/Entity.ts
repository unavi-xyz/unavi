import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

import { Quad, Triplet } from "../types";
import { BoxCollider } from "./collider/BoxCollider";
import { CylinderCollider } from "./collider/CylinderCollider";
import { SphereCollider } from "./collider/SphereCollider";
import { Collider, ColliderJSON } from "./collider/types";
import { BoxMesh } from "./mesh/BoxMesh";
import { CylinderMesh } from "./mesh/CylinderMesh";
import { GLTFMesh } from "./mesh/GLTFMesh";
import { PrimitiveMesh } from "./mesh/PrimitiveMesh";
import { SphereMesh } from "./mesh/SphereMesh";
import { Mesh, MeshJSON } from "./mesh/types";
import { Scene } from "./Scene";
import { EntityJSON } from "./types";

/*
 * Represents an entity in the scene.
 */
export class Entity {
  readonly id: string;

  scene$ = new BehaviorSubject<Scene | null>(null);

  get scene() {
    return this.scene$.value;
  }

  set scene(scene: Scene | null) {
    this.scene$.next(scene);
  }

  name$ = new BehaviorSubject("Entity");
  isInternal$ = new BehaviorSubject(false);

  parentId$ = new BehaviorSubject<string>("root");
  childrenIds$ = new BehaviorSubject<string[]>([]);

  position$ = new BehaviorSubject<Triplet>([0, 0, 0]);
  rotation$ = new BehaviorSubject<Quad>([0, 0, 0, 1]);
  scale$ = new BehaviorSubject<Triplet>([1, 1, 1]);

  mesh$ = new BehaviorSubject<Mesh | null>(null);
  materialId$ = new BehaviorSubject<string | null>(null);
  collider$ = new BehaviorSubject<Collider | null>(null);

  constructor({ id = nanoid() }: { id?: string } = {}) {
    this.id = id;
  }

  get name() {
    return this.name$.value;
  }

  set name(name: string) {
    this.name$.next(name);
  }

  get isInternal() {
    return this.isInternal$.value;
  }

  set isInternal(isInternal: boolean) {
    this.isInternal$.next(isInternal);
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

  set rotation(rotation: Quad) {
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
    if (this.mesh) this.mesh.destroy();
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

  get collider() {
    return this.collider$.value;
  }

  set collider(collider: Collider | null) {
    if (this.collider) this.collider.destroy();
    this.collider$.next(collider);
  }

  destroy() {
    this.mesh?.destroy();
    this.collider?.destroy();

    this.name$.complete();
    this.isInternal$.complete();
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
    const collider = this.collider ? this.collider.toJSON() : null;

    return {
      id: this.id,
      name: this.name,
      isInternal: this.isInternal,
      parentId: this.parentId,
      position: this.position,
      rotation: this.rotation,
      scale: this.scale,
      mesh,
      materialId: this.materialId,
      collider,
    };
  }

  applyJSON(json: Partial<EntityJSON>) {
    if (json.name !== undefined) this.name = json.name;
    if (json.isInternal !== undefined) this.isInternal = json.isInternal;

    if (json.parentId !== undefined) this.parentId = json.parentId;
    if (json.position !== undefined) this.position = json.position;
    if (json.rotation !== undefined) this.rotation = json.rotation;
    if (json.scale !== undefined) this.scale = json.scale;

    if (json.mesh !== undefined) this.mesh = createMesh(json.mesh);
    if (json.materialId !== undefined) this.materialId = json.materialId;
    if (json.collider !== undefined)
      this.collider = createCollider(json.collider);
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
    case "glTF":
      return GLTFMesh.fromJSON(json);
    case "Primitive":
      return PrimitiveMesh.fromJSON(json);
    default:
      throw new Error("Unknown mesh type");
  }
}

function createCollider(json: ColliderJSON | null) {
  if (!json) return null;

  switch (json.type) {
    case "Box":
      return BoxCollider.fromJSON(json);
    case "Sphere":
      return SphereCollider.fromJSON(json);
    case "Cylinder":
      return CylinderCollider.fromJSON(json);
    default:
      throw new Error("Unknown collider type");
  }
}

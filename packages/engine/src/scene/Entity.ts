import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

import { GameScene } from "../game/GameScene";
import { MainScene } from "../main/MainScene";
import { RenderScene } from "../render/classes/RenderScene";
import { Triplet } from "../types";
import { BoxCollider } from "./BoxCollider";
import { BoxMesh } from "./BoxMesh";
import { CylinderCollider } from "./CylinderCollider";
import { CylinderMesh } from "./CylinderMesh";
import { SphereCollider } from "./SphereCollider";
import { SphereMesh } from "./SphereMesh";
import { Collider, ColliderJSON, EntityJSON, Mesh, MeshJSON } from "./types";

/*
 * This class represents an entity in the scene.
 */
export class Entity {
  scene$ = new BehaviorSubject<MainScene | RenderScene | GameScene | null>(
    null
  );

  get scene() {
    return this.scene$.value;
  }

  set scene(scene: MainScene | RenderScene | GameScene | null) {
    this.scene$.next(scene);
  }

  readonly id: string;
  name$ = new BehaviorSubject("Entity");

  parentId$ = new BehaviorSubject<string>("root");
  childrenIds$ = new BehaviorSubject<string[]>([]);

  position$ = new BehaviorSubject<Triplet>([0, 0, 0]);
  rotation$ = new BehaviorSubject<Triplet>([0, 0, 0]);
  scale$ = new BehaviorSubject<Triplet>([1, 1, 1]);

  mesh$ = new BehaviorSubject<Mesh | null>(null);
  materialId$ = new BehaviorSubject<string | null>(null);
  collider$ = new BehaviorSubject<Collider | null>(null);

  globalPosition$ = new BehaviorSubject<Triplet>([0, 0, 0]);
  globalQuaternion$ = new BehaviorSubject<[number, number, number, number]>([
    0, 0, 0, 1,
  ]);

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
    if (this.scene instanceof GameScene) return null;
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

  get globalPosition() {
    return this.globalPosition$.value;
  }

  set globalPosition(globalPosition: Triplet) {
    this.globalPosition$.next(globalPosition);
  }

  get globalQuaternion() {
    return this.globalQuaternion$.value;
  }

  set globalQuaternion(globalQuaternion: [number, number, number, number]) {
    this.globalQuaternion$.next(globalQuaternion);
  }

  destroy() {
    this.mesh?.destroy();
    this.collider?.destroy();

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
    const collider = this.collider ? this.collider.toJSON() : null;

    return {
      id: this.id,
      parentId: this.parentId,
      name: this.name,
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

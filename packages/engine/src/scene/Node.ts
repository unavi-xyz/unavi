import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

import { Quad, Triplet } from "../types";
import { BoxCollider } from "./collider/BoxCollider";
import { CylinderCollider } from "./collider/CylinderCollider";
import { HullCollider } from "./collider/HullCollider";
import { MeshCollider } from "./collider/MeshCollider";
import { SphereCollider } from "./collider/SphereCollider";
import { Collider, ColliderJSON } from "./collider/types";
import { Scene } from "./Scene";
import { NodeJSON } from "./types";

/*
 * Represents an node in the scene.
 */
export class Node {
  readonly id: string;

  scene$ = new BehaviorSubject<Scene | null>(null);

  get scene() {
    return this.scene$.value;
  }

  set scene(scene: Scene | null) {
    this.scene$.next(scene);
  }

  name$ = new BehaviorSubject("Node");
  isInternal$ = new BehaviorSubject(false);

  parentId$ = new BehaviorSubject<string>("root");
  childrenIds$ = new BehaviorSubject<string[]>([]);

  position$ = new BehaviorSubject<Triplet>([0, 0, 0]);
  rotation$ = new BehaviorSubject<Quad>([0, 0, 0, 1]);
  scale$ = new BehaviorSubject<Triplet>([1, 1, 1]);

  meshId$ = new BehaviorSubject<string | null>(null);
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
    return this.scene?.nodes$.value[this.parentId$.value];
  }

  get parentId() {
    return this.parentId$.value;
  }

  set parentId(parentId: string) {
    // Remove from previous parent
    if (this.parent) {
      this.parent.childrenIds = this.parent.childrenIds.filter(
        (id) => id !== this.id
      );
    }

    // Set new parent
    this.parentId$.next(parentId);

    // Add to new parent
    if (this.parent)
      this.parent.childrenIds = [...this.parent.childrenIds, this.id];
  }

  get children() {
    const children = this.childrenIds$.value.map(
      (id) => this.scene?.nodes$.value[id]
    );
    const filtered = children.filter((child) => child !== undefined) as Node[];
    return filtered;
  }

  get childrenIds() {
    return this.childrenIds$.value;
  }

  set childrenIds(childrenIds: string[]) {
    this.childrenIds$.next(childrenIds);
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

  get meshId() {
    return this.meshId$.value;
  }

  set meshId(meshId: string | null) {
    this.meshId$.next(meshId);
  }

  get collider() {
    return this.collider$.value;
  }

  set collider(collider: Collider | null) {
    if (this.collider) this.collider.destroy();
    this.collider$.next(collider);
  }

  destroy() {
    this.collider?.destroy();

    this.name$.complete();
    this.isInternal$.complete();
    this.parentId$.complete();
    this.childrenIds$.complete();
    this.position$.complete();
    this.rotation$.complete();
    this.scale$.complete();
    this.meshId$.complete();
  }

  toJSON(): NodeJSON {
    const collider = this.collider ? this.collider.toJSON() : null;

    return {
      id: this.id,
      name: this.name,
      isInternal: this.isInternal,
      parentId: this.parentId,
      position: this.position,
      rotation: this.rotation,
      scale: this.scale,
      meshId: this.meshId,
      collider,
    };
  }

  applyJSON(json: Partial<NodeJSON>) {
    if (json.name !== undefined) this.name = json.name;
    if (json.isInternal !== undefined) this.isInternal = json.isInternal;

    if (json.parentId !== undefined) this.parentId = json.parentId;
    if (json.position !== undefined) this.position = json.position;
    if (json.rotation !== undefined) this.rotation = json.rotation;
    if (json.scale !== undefined) this.scale = json.scale;

    if (json.meshId !== undefined) this.meshId = json.meshId;
    if (json.collider !== undefined)
      this.collider = createCollider(json.collider);
  }

  static fromJSON(json: NodeJSON) {
    const node = new Node({ id: json.id });
    node.applyJSON(json);
    return node;
  }
}

function createCollider(json: ColliderJSON | null) {
  if (!json) return null;

  switch (json.type) {
    case "box": {
      return BoxCollider.fromJSON(json);
    }

    case "sphere": {
      return SphereCollider.fromJSON(json);
    }

    case "cylinder": {
      return CylinderCollider.fromJSON(json);
    }

    case "hull": {
      return new HullCollider();
    }

    case "mesh": {
      return new MeshCollider();
    }

    default: {
      throw new Error("Unknown collider type");
    }
  }
}

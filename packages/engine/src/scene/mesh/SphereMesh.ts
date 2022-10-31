import { BehaviorSubject } from "rxjs";

import { BaseMesh } from "./BaseMesh";
import { SphereMeshJSON } from "./types";

export class SphereMesh extends BaseMesh {
  readonly type = "Sphere";

  radius$ = new BehaviorSubject(0.5);
  widthSegments$ = new BehaviorSubject(32);
  heightSegments$ = new BehaviorSubject(32);

  constructor(id?: string) {
    super(id);
  }

  get radius() {
    return this.radius$.value;
  }

  set radius(radius: number) {
    const clamped = Math.max(0, radius);
    this.radius$.next(clamped);
  }

  get widthSegments() {
    return this.widthSegments$.value;
  }

  set widthSegments(widthSegments: number) {
    const clamped = Math.max(3, widthSegments);
    const rounded = Math.round(clamped);
    this.widthSegments$.next(rounded);
  }

  get heightSegments() {
    return this.heightSegments$.value;
  }

  set heightSegments(heightSegments: number) {
    const clamped = Math.max(3, heightSegments);
    const rounded = Math.round(clamped);
    this.heightSegments$.next(rounded);
  }

  destroy() {
    this.materialId$.complete();
    this.radius$.complete();
    this.widthSegments$.complete();
    this.heightSegments$.complete();
  }

  toJSON(): SphereMeshJSON {
    return {
      id: this.id,
      materialId: this.materialId,
      type: this.type,
      radius: this.radius$.value,
      widthSegments: this.widthSegments$.value,
      heightSegments: this.heightSegments$.value,
    };
  }

  applyJSON(json: Partial<SphereMeshJSON>) {
    if (json.materialId !== undefined) this.materialId = json.materialId;
    if (json.radius !== undefined) this.radius = json.radius;
    if (json.widthSegments !== undefined)
      this.widthSegments = json.widthSegments;
    if (json.heightSegments !== undefined)
      this.heightSegments = json.heightSegments;
  }

  static fromJSON(json: SphereMeshJSON) {
    const mesh = new SphereMesh(json.id);
    mesh.applyJSON(json);
    return mesh;
  }
}

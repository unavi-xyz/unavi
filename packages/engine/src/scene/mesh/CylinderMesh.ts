import { BehaviorSubject } from "rxjs";

import { CylinderMeshJSON } from "./types";

export class CylinderMesh {
  readonly type = "Cylinder";

  radius$ = new BehaviorSubject(0.5);
  height$ = new BehaviorSubject(1);
  radialSegments$ = new BehaviorSubject(32);

  get radius() {
    return this.radius$.value;
  }

  set radius(radius: number) {
    const clamped = Math.max(0, radius);
    this.radius$.next(clamped);
  }

  get height() {
    return this.height$.value;
  }

  set height(height: number) {
    const clamped = Math.max(0, height);
    this.height$.next(clamped);
  }

  get radialSegments() {
    return this.radialSegments$.value;
  }

  set radialSegments(radialSegments: number) {
    const clamped = Math.max(3, radialSegments);
    const rounded = Math.round(clamped);
    this.radialSegments$.next(rounded);
  }

  destroy() {
    this.radius$.complete();
    this.height$.complete();
    this.radialSegments$.complete();
  }

  toJSON(): CylinderMeshJSON {
    return {
      type: this.type,
      radius: this.radius$.value,
      height: this.height$.value,
      radialSegments: this.radialSegments$.value,
    };
  }

  applyJSON(json: Partial<CylinderMeshJSON>) {
    if (json.radius !== undefined) this.radius = json.radius;
    if (json.height !== undefined) this.height = json.height;
    if (json.radialSegments !== undefined)
      this.radialSegments = json.radialSegments;
  }

  static fromJSON(json: CylinderMeshJSON) {
    const mesh = new CylinderMesh();
    mesh.applyJSON(json);
    return mesh;
  }
}

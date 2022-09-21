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
    this.radius$.next(radius);
  }

  get height() {
    return this.height$.value;
  }

  set height(height: number) {
    this.height$.next(height);
  }

  get radialSegments() {
    return this.radialSegments$.value;
  }

  set radialSegments(radialSegments: number) {
    this.radialSegments$.next(radialSegments);
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
    if (json.radius) this.radius = json.radius;
    if (json.height) this.height = json.height;
    if (json.radialSegments) this.radialSegments = json.radialSegments;
  }

  static fromJSON(json: CylinderMeshJSON) {
    const mesh = new CylinderMesh();
    mesh.applyJSON(json);
    return mesh;
  }
}

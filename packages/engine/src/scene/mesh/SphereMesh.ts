import { BehaviorSubject } from "rxjs";

import { SphereMeshJSON } from "./types";

export class SphereMesh {
  readonly type = "Sphere";

  radius$ = new BehaviorSubject(0.5);
  widthSegments$ = new BehaviorSubject(32);
  heightSegments$ = new BehaviorSubject(32);

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
    this.radius$.complete();
    this.widthSegments$.complete();
    this.heightSegments$.complete();
  }

  toJSON(): SphereMeshJSON {
    return {
      type: this.type,
      radius: this.radius$.value,
      widthSegments: this.widthSegments$.value,
      heightSegments: this.heightSegments$.value,
    };
  }

  applyJSON(json: Partial<SphereMeshJSON>) {
    if (json.radius !== undefined) this.radius = json.radius;
    if (json.widthSegments !== undefined)
      this.widthSegments = json.widthSegments;
    if (json.heightSegments !== undefined)
      this.heightSegments = json.heightSegments;
  }

  static fromJSON(json: SphereMeshJSON) {
    const mesh = new SphereMesh();
    mesh.applyJSON(json);
    return mesh;
  }
}

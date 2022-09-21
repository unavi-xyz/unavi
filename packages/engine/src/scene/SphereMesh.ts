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
    this.radius$.next(radius);
  }

  get widthSegments() {
    return this.widthSegments$.value;
  }

  set widthSegments(widthSegments: number) {
    this.widthSegments$.next(widthSegments);
  }

  get heightSegments() {
    return this.heightSegments$.value;
  }

  set heightSegments(heightSegments: number) {
    this.heightSegments$.next(heightSegments);
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
    if (json.radius) this.radius = json.radius;
    if (json.widthSegments) this.widthSegments = json.widthSegments;
    if (json.heightSegments) this.heightSegments = json.heightSegments;
  }

  static fromJSON(json: SphereMeshJSON) {
    const mesh = new SphereMesh();
    mesh.applyJSON(json);
    return mesh;
  }
}

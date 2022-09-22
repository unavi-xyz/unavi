import { BehaviorSubject } from "rxjs";

import { BoxMeshJSON } from "./types";

export class BoxMesh {
  readonly type = "Box";

  width$ = new BehaviorSubject(1);
  height$ = new BehaviorSubject(1);
  depth$ = new BehaviorSubject(1);

  get width() {
    return this.width$.value;
  }

  set width(width: number) {
    const clamped = Math.max(0, width);
    this.width$.next(clamped);
  }

  get height() {
    return this.height$.value;
  }

  set height(height: number) {
    const clamped = Math.max(0, height);
    this.height$.next(clamped);
  }

  get depth() {
    return this.depth$.value;
  }

  set depth(depth: number) {
    const clamped = Math.max(0, depth);
    this.depth$.next(clamped);
  }

  toJSON(): BoxMeshJSON {
    return {
      type: this.type,
      width: this.width$.value,
      height: this.height$.value,
      depth: this.depth$.value,
    };
  }

  applyJSON(json: Partial<BoxMeshJSON>) {
    if (json.width) this.width = json.width;
    if (json.height) this.height = json.height;
    if (json.depth) this.depth = json.depth;
  }

  static fromJSON(json: BoxMeshJSON) {
    const mesh = new BoxMesh();
    mesh.applyJSON(json);
    return mesh;
  }
}

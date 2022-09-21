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
    this.width$.next(width);
  }

  get height() {
    return this.height$.value;
  }

  set height(height: number) {
    this.height$.next(height);
  }

  get depth() {
    return this.depth$.value;
  }

  set depth(depth: number) {
    this.depth$.next(depth);
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

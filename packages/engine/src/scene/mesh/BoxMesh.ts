import { BehaviorSubject } from "rxjs";

import { BaseMesh } from "./BaseMesh";
import { BoxMeshJSON } from "./types";

export class BoxMesh extends BaseMesh {
  readonly type = "Box";

  width$ = new BehaviorSubject(1);
  height$ = new BehaviorSubject(1);
  depth$ = new BehaviorSubject(1);

  constructor(id?: string) {
    super(id);
  }

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

  destroy() {
    this.materialId$.complete();
    this.width$.complete();
    this.height$.complete();
    this.depth$.complete();
  }

  toJSON(): BoxMeshJSON {
    return {
      id: this.id,
      materialId: this.materialId,
      type: this.type,
      width: this.width,
      height: this.height,
      depth: this.depth,
    };
  }

  applyJSON(json: Partial<BoxMeshJSON>) {
    if (json.materialId !== undefined) this.materialId = json.materialId;
    if (json.width !== undefined) this.width = json.width;
    if (json.height !== undefined) this.height = json.height;
    if (json.depth !== undefined) this.depth = json.depth;
  }

  static fromJSON(json: BoxMeshJSON) {
    const mesh = new BoxMesh(json.id);
    mesh.applyJSON(json);
    return mesh;
  }
}

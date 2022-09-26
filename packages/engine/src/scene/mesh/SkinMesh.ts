import { BehaviorSubject } from "rxjs";

import { SkinMeshJSON } from "./types";

export class SkinMesh {
  readonly type = "Skin";

  inverseBindMatricesId$ = new BehaviorSubject<string | null>(null);

  get inverseBindMatricesId() {
    return this.inverseBindMatricesId$.value;
  }

  set inverseBindMatricesId(inverseBindMatricesId: string | null) {
    this.inverseBindMatricesId$.next(inverseBindMatricesId);
  }

  destroy() {
    this.inverseBindMatricesId$.complete();
  }

  toJSON(): SkinMeshJSON {
    return {
      type: this.type,
      inverseBindMatricesId: this.inverseBindMatricesId,
    };
  }

  applyJSON(json: Partial<SkinMeshJSON>) {
    if (json.inverseBindMatricesId !== undefined)
      this.inverseBindMatricesId = json.inverseBindMatricesId;
  }

  static fromJSON(json: SkinMeshJSON) {
    const mesh = new SkinMesh();
    mesh.applyJSON(json);
    return mesh;
  }
}

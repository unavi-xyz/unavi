import { BehaviorSubject } from "rxjs";

import { BaseMesh } from "./BaseMesh";
import { GLTFMeshJSON } from "./types";

export class GLTFMesh extends BaseMesh {
  readonly type = "glTF";

  uri$ = new BehaviorSubject<string | null>(null);

  constructor(id?: string) {
    super(id);
  }

  get name() {
    return this.name$.value;
  }

  set name(name: string | null) {
    this.name$.next(name);
  }

  get uri() {
    return this.uri$.value;
  }

  set uri(uri: string | null) {
    this.uri$.next(uri);
  }

  destroy() {
    this.name$.complete();
    this.materialId$.complete();
    this.uri$.complete();
  }

  toJSON(): GLTFMeshJSON {
    return {
      id: this.id,
      materialId: this.materialId,
      type: this.type,
      name: this.name,
      uri: this.uri,
    };
  }

  applyJSON(json: Partial<GLTFMeshJSON>) {
    if (json.materialId !== undefined) this.materialId = json.materialId;
    if (json.name !== undefined) this.name = json.name;
    if (json.uri !== undefined) this.uri = json.uri;
  }

  static fromJSON(json: GLTFMeshJSON) {
    const mesh = new GLTFMesh(json.id);
    mesh.applyJSON(json);
    return mesh;
  }
}

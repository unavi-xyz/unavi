import { BehaviorSubject } from "rxjs";

import { GLTFMeshJSON } from "./types";

export class GLTFMesh {
  readonly type = "glTF";

  uri$ = new BehaviorSubject<string | null>(null);

  get uri() {
    return this.uri$.value;
  }

  set uri(uri: string | null) {
    this.uri$.next(uri);
  }

  destroy() {
    this.uri$.complete();
  }

  toJSON(): GLTFMeshJSON {
    return {
      type: this.type,
      uri: this.uri,
    };
  }

  applyJSON(json: Partial<GLTFMeshJSON>) {
    if (json.uri !== undefined) this.uri = json.uri;
  }

  static fromJSON(json: GLTFMeshJSON) {
    const mesh = new GLTFMesh();
    mesh.applyJSON(json);
    return mesh;
  }
}

import { BehaviorSubject } from "rxjs";

import { GLTFMeshJSON } from "./types";

export class GLTFMesh {
  readonly type = "glTF";

  name$ = new BehaviorSubject<string | null>(null);
  uri$ = new BehaviorSubject<string | null>(null);

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
    this.uri$.complete();
  }

  toJSON(): GLTFMeshJSON {
    return {
      type: this.type,
      name: this.name,
      uri: this.uri,
    };
  }

  applyJSON(json: Partial<GLTFMeshJSON>) {
    if (json.name !== undefined) this.name = json.name;
    if (json.uri !== undefined) this.uri = json.uri;
  }

  static fromJSON(json: GLTFMeshJSON) {
    const mesh = new GLTFMesh();
    mesh.applyJSON(json);
    return mesh;
  }
}

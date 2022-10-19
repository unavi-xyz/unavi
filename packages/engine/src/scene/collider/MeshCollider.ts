import { MeshColliderJSON } from "./types";

export class MeshCollider {
  readonly type = "mesh";

  destroy() {}

  toJSON(): MeshColliderJSON {
    return {
      type: this.type,
    };
  }
}

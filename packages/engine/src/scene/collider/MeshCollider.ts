import { MeshColliderJSON } from "./types";

export class MeshCollider {
  readonly type = "mesh";

  meshId: string;

  constructor(meshId: string) {
    this.meshId = meshId;
  }

  destroy() {}

  toJSON(): MeshColliderJSON {
    return {
      type: this.type,
      meshId: this.meshId,
    };
  }
}

import { MeshColliderJSON } from "./types";

export class MeshCollider {
  readonly type = "mesh";

  meshId: string | null = null;

  destroy() {}

  toJSON(): MeshColliderJSON {
    return {
      type: this.type,
      meshId: this.meshId,
    };
  }

  applyJSON(json: Partial<MeshColliderJSON>) {
    if (json.meshId !== undefined) this.meshId = json.meshId;
  }

  static fromJSON(json: MeshColliderJSON) {
    const collider = new MeshCollider();
    collider.applyJSON(json);
    return collider;
  }
}

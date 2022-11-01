import { HullColliderJSON } from "./types";

export class HullCollider {
  readonly type = "hull";

  meshId: string | null = null;

  destroy() {}

  toJSON(): HullColliderJSON {
    return {
      type: this.type,
      meshId: this.meshId,
    };
  }

  applyJSON(json: Partial<HullColliderJSON>) {
    if (json.meshId !== undefined) this.meshId = json.meshId;
  }

  static fromJSON(json: HullColliderJSON) {
    const collider = new HullCollider();
    collider.applyJSON(json);
    return collider;
  }
}

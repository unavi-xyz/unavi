import { HullColliderJSON } from "./types";

export class HullCollider {
  readonly type = "hull";

  meshId: string;

  constructor(meshId: string) {
    this.meshId = meshId;
  }

  destroy() {}

  toJSON(): HullColliderJSON {
    return {
      type: this.type,
      meshId: this.meshId,
    };
  }
}

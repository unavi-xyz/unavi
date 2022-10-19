import { HullColliderJSON } from "./types";

export class HullCollider {
  readonly type = "hull";

  destroy() {}

  toJSON(): HullColliderJSON {
    return {
      type: this.type,
    };
  }
}

import { AutoColliderJSON } from "./types";

export class AutoCollider {
  readonly type = "auto";

  destroy() {}

  toJSON(): AutoColliderJSON {
    return {
      type: this.type,
    };
  }
}

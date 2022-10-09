import { BehaviorSubject } from "rxjs";

import { Triplet } from "../../types";
import { BoxColliderJSON } from "./types";

export class BoxCollider {
  readonly type = "box";
  size$ = new BehaviorSubject<Triplet>([1, 1, 1]);

  get size() {
    return this.size$.value;
  }

  set size(size: Triplet) {
    this.size$.next(size);
  }

  destroy() {
    this.size$.complete();
  }

  toJSON(): BoxColliderJSON {
    return {
      type: this.type,
      size: this.size,
    };
  }

  applyJSON(json: Partial<BoxColliderJSON>) {
    if (json.size !== undefined) this.size = json.size;
  }

  static fromJSON(json: BoxColliderJSON) {
    const collider = new BoxCollider();
    collider.applyJSON(json);
    return collider;
  }
}

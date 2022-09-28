import { BehaviorSubject } from "rxjs";

import { SphereColliderJSON } from "./types";

export class SphereCollider {
  readonly type = "Sphere";
  radius$ = new BehaviorSubject(0.5);

  get radius() {
    return this.radius$.value;
  }

  set radius(radius: number) {
    this.radius$.next(radius);
  }

  destroy() {
    this.radius$.complete();
  }

  toJSON(): SphereColliderJSON {
    return {
      type: this.type,
      radius: this.radius,
    };
  }

  applyJSON(json: Partial<SphereColliderJSON>) {
    if (json.radius !== undefined) this.radius = json.radius;
  }

  static fromJSON(json: SphereColliderJSON) {
    const collider = new SphereCollider();
    collider.applyJSON(json);
    return collider;
  }
}

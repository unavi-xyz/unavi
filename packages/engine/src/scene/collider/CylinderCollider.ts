import { BehaviorSubject } from "rxjs";

import { CylinderColliderJSON } from "./types";

export class CylinderCollider {
  readonly type = "Cylinder";
  radius$ = new BehaviorSubject(0.5);
  height$ = new BehaviorSubject(1);

  get radius() {
    return this.radius$.value;
  }

  set radius(radius: number) {
    this.radius$.next(radius);
  }

  get height() {
    return this.height$.value;
  }

  set height(height: number) {
    this.height$.next(height);
  }

  destroy() {
    this.radius$.complete();
    this.height$.complete();
  }

  toJSON(): CylinderColliderJSON {
    return {
      type: this.type,
      radius: this.radius,
      height: this.height,
    };
  }

  applyJSON(json: Partial<CylinderColliderJSON>) {
    if (json.radius !== undefined) this.radius = json.radius;
    if (json.height !== undefined) this.height = json.height;
  }

  static fromJSON(json: CylinderColliderJSON) {
    const collider = new CylinderCollider();
    collider.applyJSON(json);
    return collider;
  }
}

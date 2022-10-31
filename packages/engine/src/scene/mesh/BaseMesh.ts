import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

export class BaseMesh {
  id = nanoid();
  isInternal = false;

  name$ = new BehaviorSubject<string | null>(null);
  materialId$ = new BehaviorSubject<string | null>(null);

  constructor(id?: string) {
    if (id) this.id = id;
  }

  get name() {
    return this.name$.value;
  }

  set name(name: string | null) {
    this.name$.next(name);
  }

  get materialId() {
    return this.materialId$.value;
  }

  set materialId(materialId: string | null) {
    this.materialId$.next(materialId);
  }
}

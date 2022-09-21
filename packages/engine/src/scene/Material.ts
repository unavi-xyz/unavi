import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

import { Triplet } from "../types";
import { MaterialJSON } from "./types";

export class Material {
  id: string;
  name$ = new BehaviorSubject("New Material");
  color$ = new BehaviorSubject<Triplet>([1, 1, 1]);
  roughness$ = new BehaviorSubject(1);
  metalness$ = new BehaviorSubject(0);

  constructor({ id }: { id?: string } = {}) {
    this.id = id ?? nanoid();
  }

  get name() {
    return this.name$.value;
  }
  set name(name: string) {
    this.name$.next(name);
  }

  get color() {
    return this.color$.value;
  }
  set color(color: Triplet) {
    this.color$.next(color);
  }

  get roughness() {
    return this.roughness$.value;
  }
  set roughness(roughness: number) {
    this.roughness$.next(roughness);
  }

  get metalness() {
    return this.metalness$.value;
  }
  set metalness(metalness: number) {
    this.metalness$.next(metalness);
  }

  toJSON(): MaterialJSON {
    return {
      id: this.id,
      name: this.name,
      color: this.color,
      roughness: this.roughness,
      metalness: this.metalness,
    };
  }

  applyJSON(json: Partial<MaterialJSON>) {
    if (json.name !== undefined) this.name = json.name;
    if (json.color !== undefined) this.color = json.color;
    if (json.roughness !== undefined) this.roughness = json.roughness;
    if (json.metalness !== undefined) this.metalness = json.metalness;
  }

  static fromJSON(json: MaterialJSON) {
    const material = new Material({ id: json.id });
    material.applyJSON(json);
    return material;
  }
}

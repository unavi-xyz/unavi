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
    const clamped: Triplet = [
      Math.max(0, Math.min(1, color[0])),
      Math.max(0, Math.min(1, color[1])),
      Math.max(0, Math.min(1, color[2])),
    ];
    this.color$.next(clamped);
  }

  get roughness() {
    return this.roughness$.value;
  }

  set roughness(roughness: number) {
    const clamped = Math.max(0, Math.min(1, roughness));
    this.roughness$.next(clamped);
  }

  get metalness() {
    return this.metalness$.value;
  }

  set metalness(metalness: number) {
    const clamped = Math.max(0, Math.min(1, metalness));
    this.metalness$.next(clamped);
  }

  destroy() {
    this.name$.complete();
    this.color$.complete();
    this.roughness$.complete();
    this.metalness$.complete();
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

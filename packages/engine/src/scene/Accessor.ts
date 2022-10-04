import { TypedArray } from "@gltf-transform/core";
import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

import { AccessorJSON, AccessorType } from "./types";

export class Accessor {
  readonly id: string;

  isInternal$ = new BehaviorSubject(false);

  array: TypedArray;
  elementSize: number;
  type: AccessorType;
  normalized: boolean;

  constructor({
    id = nanoid(),
    array,
    elementSize,
    type,
    normalized,
  }: {
    id?: string;
    array: TypedArray;
    elementSize: number;
    type: AccessorType;
    normalized: boolean;
  }) {
    this.id = id;
    this.array = array;
    this.elementSize = elementSize;
    this.type = type;
    this.normalized = normalized;
  }

  get isInternal() {
    return this.isInternal$.value;
  }

  set isInternal(isInternal: boolean) {
    this.isInternal$.next(isInternal);
  }

  destroy() {
    this.isInternal$.complete();
  }

  toJSON(): AccessorJSON {
    return {
      id: this.id,
      isInternal: this.isInternal,
      array: this.array,
      elementSize: this.elementSize,
      type: this.type,
      normalized: this.normalized,
    };
  }

  applyJSON(json: Partial<AccessorJSON>) {
    if (json.isInternal !== undefined) this.isInternal = json.isInternal;
  }

  static fromJSON(json: AccessorJSON) {
    const accessor = new Accessor({
      id: json.id,
      array: json.array,
      elementSize: json.elementSize,
      type: json.type,
      normalized: json.normalized,
    });
    accessor.applyJSON(json);
    return accessor;
  }
}

import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

import { AnimationChannel, AnimationJSON } from "./types";

export class Animation {
  readonly id: string;

  name$ = new BehaviorSubject("New Animation");
  isInternal$ = new BehaviorSubject(false);
  channels$ = new BehaviorSubject<AnimationChannel[]>([]);

  constructor({ id = nanoid() }: { id?: string } = {}) {
    this.id = id;
  }

  get name() {
    return this.name$.value;
  }

  set name(name: string) {
    this.name$.next(name);
  }

  get isInternal() {
    return this.isInternal$.value;
  }

  set isInternal(isInternal: boolean) {
    this.isInternal$.next(isInternal);
  }

  get channels() {
    return this.channels$.value;
  }

  set channels(channels: AnimationChannel[]) {
    this.channels$.next(channels);
  }

  destroy() {
    this.name$.complete();
    this.isInternal$.complete();
    this.channels$.complete();
  }

  toJSON(): AnimationJSON {
    return {
      id: this.id,
      name: this.name,
      isInternal: this.isInternal,
      channels: this.channels,
    };
  }

  applyJSON(json: Partial<AnimationJSON>) {
    if (json.name !== undefined) this.name = json.name;
    if (json.isInternal !== undefined) this.isInternal = json.isInternal;
    if (json.channels !== undefined) this.channels = json.channels;
  }

  static fromJSON(json: AnimationJSON) {
    const texture = new Animation({ id: json.id });
    texture.applyJSON(json);
    return texture;
  }
}

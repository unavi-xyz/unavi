import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

import { TextureJSON } from "./types";

export class Texture {
  readonly id: string;

  name$ = new BehaviorSubject("New Texture");
  isInternal$ = new BehaviorSubject(false);

  sourceId$ = new BehaviorSubject<string | null>(null);

  magFilter$ = new BehaviorSubject<number>(9729);
  minFIlters$ = new BehaviorSubject<number>(9987);
  wrapS$ = new BehaviorSubject<number>(0);
  wrapT$ = new BehaviorSubject<number>(0);

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

  get sourceId() {
    return this.sourceId$.value;
  }

  set sourceId(image: string | null) {
    this.sourceId$.next(image);
  }

  get magFilter() {
    return this.magFilter$.value;
  }

  set magFilter(magFilter: number) {
    this.magFilter$.next(magFilter);
  }

  get minFilter() {
    return this.minFIlters$.value;
  }

  set minFilter(minFilter: number) {
    this.minFIlters$.next(minFilter);
  }

  get wrapS() {
    return this.wrapS$.value;
  }

  set wrapS(wrapS: number) {
    this.wrapS$.next(wrapS);
  }

  get wrapT() {
    return this.wrapT$.value;
  }

  set wrapT(wrapT: number) {
    this.wrapT$.next(wrapT);
  }

  destroy() {
    this.name$.complete();
    this.isInternal$.complete();
    this.sourceId$.complete();
    this.magFilter$.complete();
    this.minFIlters$.complete();
    this.wrapS$.complete();
    this.wrapT$.complete();
  }

  toJSON(): TextureJSON {
    return {
      id: this.id,
      name: this.name,
      isInternal: this.isInternal,
      sourceId: this.sourceId,
      magFilter: this.magFilter,
      minFilter: this.minFilter,
      wrapS: this.wrapS,
      wrapT: this.wrapT,
    };
  }

  applyJSON(json: Partial<TextureJSON>) {
    if (json.name !== undefined) this.name = json.name;
    if (json.isInternal !== undefined) this.isInternal = json.isInternal;
    if (json.sourceId !== undefined) this.sourceId = json.sourceId;
    if (json.magFilter !== undefined) this.magFilter = json.magFilter;
    if (json.minFilter !== undefined) this.minFilter = json.minFilter;
    if (json.wrapS !== undefined) this.wrapS = json.wrapS;
    if (json.wrapT !== undefined) this.wrapT = json.wrapT;
  }

  static fromJSON(json: TextureJSON) {
    const texture = new Texture({ id: json.id });
    texture.applyJSON(json);
    return texture;
  }
}

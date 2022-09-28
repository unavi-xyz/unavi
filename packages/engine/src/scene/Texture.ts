import { BehaviorSubject } from "rxjs";

import { TextureJSON } from "./types";

export class Texture {
  imageId$ = new BehaviorSubject<string | null>(null);
  magFilter$ = new BehaviorSubject<number>(9729);
  minFIlters$ = new BehaviorSubject<number>(9987);
  wrapS$ = new BehaviorSubject<number>(0);
  wrapT$ = new BehaviorSubject<number>(0);

  get imageId() {
    return this.imageId$.value;
  }

  set imageId(image: string | null) {
    this.imageId$.next(image);
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
    this.imageId$.complete();
    this.magFilter$.complete();
    this.minFIlters$.complete();
    this.wrapS$.complete();
    this.wrapT$.complete();
  }

  toJSON(): TextureJSON {
    return {
      imageId: this.imageId,
      magFilter: this.magFilter,
      minFilter: this.minFilter,
      wrapS: this.wrapS,
      wrapT: this.wrapT,
    };
  }

  applyJSON(json: Partial<TextureJSON>) {
    if (json.imageId !== undefined) this.imageId = json.imageId;
    if (json.magFilter !== undefined) this.magFilter = json.magFilter;
    if (json.minFilter !== undefined) this.minFilter = json.minFilter;
    if (json.wrapS !== undefined) this.wrapS = json.wrapS;
    if (json.wrapT !== undefined) this.wrapT = json.wrapT;
  }

  static fromJSON(json: TextureJSON) {
    const texture = new Texture();
    texture.applyJSON(json);
    return texture;
  }
}

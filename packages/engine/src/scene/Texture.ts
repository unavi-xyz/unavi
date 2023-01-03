import { GLTF } from "@gltf-transform/core";
import { BehaviorSubject } from "rxjs";

import { TextureJSON } from "./types";

export class Texture {
  name$ = new BehaviorSubject<string>("New Texture");

  imageId$ = new BehaviorSubject<string | null>(null);
  magFilter$ = new BehaviorSubject<GLTF.TextureMagFilter>(9729);
  minFilters$ = new BehaviorSubject<GLTF.TextureMinFilter>(9987);
  wrapS$ = new BehaviorSubject<GLTF.TextureWrapMode>(33071);
  wrapT$ = new BehaviorSubject<GLTF.TextureWrapMode>(33071);

  offset: [number, number] | null = null;
  rotation: number | null = null;
  scale: [number, number] | null = null;

  get name() {
    return this.name$.value;
  }

  set name(name: string) {
    this.name$.next(name);
  }

  get imageId() {
    return this.imageId$.value;
  }

  set imageId(image: string | null) {
    this.imageId$.next(image);
  }

  get magFilter() {
    return this.magFilter$.value;
  }

  set magFilter(magFilter: GLTF.TextureMagFilter) {
    this.magFilter$.next(magFilter);
  }

  get minFilter() {
    return this.minFilters$.value;
  }

  set minFilter(minFilter: GLTF.TextureMinFilter) {
    this.minFilters$.next(minFilter);
  }

  get wrapS() {
    return this.wrapS$.value;
  }

  set wrapS(wrapS: GLTF.TextureWrapMode) {
    this.wrapS$.next(wrapS);
  }

  get wrapT() {
    return this.wrapT$.value;
  }

  set wrapT(wrapT: GLTF.TextureWrapMode) {
    this.wrapT$.next(wrapT);
  }

  destroy() {
    this.imageId$.complete();
    this.magFilter$.complete();
    this.minFilters$.complete();
    this.wrapS$.complete();
    this.wrapT$.complete();
  }

  toJSON(): TextureJSON {
    return {
      name: this.name,
      imageId: this.imageId,
      magFilter: this.magFilter,
      minFilter: this.minFilter,
      wrapS: this.wrapS,
      wrapT: this.wrapT,
      offset: this.offset,
      rotation: this.rotation,
      scale: this.scale,
    };
  }

  applyJSON(json: Partial<TextureJSON>) {
    if (json.name !== undefined) this.name = json.name;
    if (json.imageId !== undefined) this.imageId = json.imageId;
    if (json.magFilter !== undefined) this.magFilter = json.magFilter;
    if (json.minFilter !== undefined) this.minFilter = json.minFilter;
    if (json.wrapS !== undefined) this.wrapS = json.wrapS;
    if (json.wrapT !== undefined) this.wrapT = json.wrapT;
    if (json.offset !== undefined) this.offset = json.offset;
    if (json.rotation !== undefined) this.rotation = json.rotation;
    if (json.scale !== undefined) this.scale = json.scale;
  }

  static fromJSON(json: TextureJSON) {
    const texture = new Texture();
    texture.applyJSON(json);
    return texture;
  }
}

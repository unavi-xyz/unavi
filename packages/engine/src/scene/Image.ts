import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

import { ImageJSON } from "./types";

export class Image {
  readonly id: string;

  array: Uint8Array;
  bitmap: ImageBitmap;
  mimeType: string;
  isInternal$ = new BehaviorSubject(false);

  constructor({
    id = nanoid(),
    array,
    bitmap,
    mimeType,
  }: {
    id?: string;
    array: Uint8Array;
    bitmap: ImageBitmap;
    mimeType: string;
  }) {
    this.id = id;
    this.array = array;
    this.bitmap = bitmap;
    this.mimeType = mimeType;
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

  toJSON(): ImageJSON {
    return {
      id: this.id,
      isInternal: this.isInternal,
      array: this.array,
      bitmap: this.bitmap,
      mimeType: this.mimeType,
    };
  }

  applyJSON(json: Partial<ImageJSON>) {
    if (json.isInternal !== undefined) this.isInternal = json.isInternal;
  }

  static fromJSON(json: ImageJSON) {
    const texture = new Image({
      id: json.id,
      array: json.array,
      bitmap: json.bitmap,
      mimeType: json.mimeType,
    });
    texture.applyJSON(json);
    return texture;
  }
}

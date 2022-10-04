import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

import { ImageJSON } from "./types";

export class Image {
  readonly id: string;

  bitmap: ImageBitmap;
  mimeType: string;
  isInternal$ = new BehaviorSubject(false);

  constructor({
    id = nanoid(),
    bitmap,
    mimeType,
  }: {
    id?: string;
    bitmap: ImageBitmap;
    mimeType: string;
  }) {
    this.id = id;
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
      bitmap: json.bitmap,
      mimeType: json.mimeType,
    });
    texture.applyJSON(json);
    return texture;
  }
}

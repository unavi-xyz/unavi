import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

import { ImageJSON } from "./types";

export class Image {
  readonly id: string;

  bitmap: ImageBitmap;
  isInternal$ = new BehaviorSubject(false);

  constructor({ id = nanoid(), bitmap }: { id?: string; bitmap: ImageBitmap }) {
    this.id = id;
    this.bitmap = bitmap;
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
    };
  }

  applyJSON(json: Partial<ImageJSON>) {
    if (json.isInternal !== undefined) this.isInternal = json.isInternal;
  }

  static fromJSON(json: ImageJSON) {
    const texture = new Image({ id: json.id, bitmap: json.bitmap });
    texture.applyJSON(json);
    return texture;
  }
}

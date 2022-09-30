import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

import { FileJSON } from "./types";

export class File {
  readonly id: string;

  isInternal$ = new BehaviorSubject<boolean>(false);
  name$ = new BehaviorSubject<string>("File");
  uri$ = new BehaviorSubject<string | null>(null);

  constructor({ id = nanoid() }: { id?: string } = {}) {
    this.id = id;
  }

  get isInternal() {
    return this.isInternal$.value;
  }

  set isInternal(isInternal: boolean) {
    this.isInternal$.next(isInternal);
  }

  get name() {
    return this.name$.value;
  }

  set name(name: string) {
    this.name$.next(name);
  }

  get uri() {
    return this.uri$.value;
  }

  set uri(uri: string | null) {
    this.uri$.next(uri);
  }

  destroy() {
    this.name$.complete();
    this.uri$.complete();
  }

  toJSON(): FileJSON {
    return {
      id: this.id,
      isInternal: this.isInternal,
      name: this.name,
      uri: this.uri,
    };
  }

  applyJSON(json: Partial<FileJSON>) {
    if (json.isInternal !== undefined) this.isInternal = json.isInternal;
    if (json.name !== undefined) this.name = json.name;
    if (json.uri !== undefined) this.uri = json.uri;
  }

  static fromJSON(json: FileJSON) {
    const texture = new File({ id: json.id });
    texture.applyJSON(json);
    return texture;
  }
}

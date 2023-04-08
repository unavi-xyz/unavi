import { TemplatedApp } from "uWebSockets.js";

import { Space } from "./Space";

export class SpaceRegistry {
  readonly server: TemplatedApp;

  #store = new Map<string, Space>();

  constructor(server: TemplatedApp) {
    this.server = server;
  }

  getSpace(uri: string) {
    return this.#store.get(uri);
  }

  getOrCreateSpace(uri: string) {
    const space = this.#store.get(uri);
    if (space) return space;

    const newSpace = new Space(uri, this);
    this.#store.set(uri, newSpace);
    return newSpace;
  }

  removeSpace(uri: string) {
    this.#store.delete(uri);
  }
}

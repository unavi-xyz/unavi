import { TemplatedApp } from "uWebSockets.js";

import { Space } from "./Space";

export class SpaceRegistry {
  readonly server: TemplatedApp;

  #store = new Map<number, Space>();

  constructor(server: TemplatedApp) {
    this.server = server;
  }

  getSpace(spaceId: number) {
    return this.#store.get(spaceId);
  }

  getOrCreateSpace(spaceId: number) {
    const space = this.#store.get(spaceId);
    if (space) return space;

    const newSpace = new Space(spaceId, this);
    this.#store.set(spaceId, newSpace);
    return newSpace;
  }

  removeSpace(spaceId: number) {
    this.#store.delete(spaceId);
  }
}

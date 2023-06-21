import { TemplatedApp } from "uWebSockets.js";

import { World } from "./World";

export class WorldRegistry {
  readonly server: TemplatedApp;

  #store = new Map<string, World>();

  constructor(server: TemplatedApp) {
    this.server = server;
  }

  getWorld(uri: string) {
    return this.#store.get(uri);
  }

  getOrCreateWorld(uri: string) {
    const world = this.#store.get(uri);
    if (world) return world;

    const newWorld = new World(uri, this);
    this.#store.set(uri, newWorld);
    return newWorld;
  }

  removeWorld(uri: string) {
    this.#store.delete(uri);
  }
}

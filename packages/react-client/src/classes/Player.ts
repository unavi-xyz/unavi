import { Engine } from "engine";

import { toHex } from "../utils/toHex";

/**
 * Used to store information about a player as it comes in from the server.
 * Once the engine is provided, the player will be added to the engine.
 */
export class Player {
  readonly id: number;
  #engine: Engine | null = null;

  #avatar: string | null = null;
  #handle: string | null = null;
  #name: string | null = null;
  #grounded = false;

  #displayName = "";

  constructor(id: number) {
    this.id = id;
  }

  get handle() {
    return this.#handle;
  }

  set handle(handle: string | null) {
    this.#handle = handle;
    this.#updateDisplayName();
  }

  get avatar() {
    return this.#avatar;
  }

  set avatar(avatar: string | null) {
    this.#avatar = avatar;

    if (this.#engine) {
      const player = this.#engine.player.getPlayer(this.id);
      if (player) player.avatar = avatar;
    }
  }

  get displayName() {
    return this.#displayName;
  }

  get engine() {
    return this.#engine;
  }

  set engine(engine: Engine | null) {
    this.#engine = engine;
    if (!engine) return;

    const player = engine.player.getPlayer(this.id) || engine.player.addPlayer(this.id);

    player.name = this.displayName;
    player.avatar = this.avatar;
    player.grounded = this.grounded;
  }

  get grounded() {
    return this.#grounded;
  }

  set grounded(grounded: boolean) {
    this.#grounded = grounded;

    if (this.#engine) {
      const player = this.#engine.player.getPlayer(this.id);
      if (player) player.grounded = grounded;
    }
  }

  get name() {
    return this.#name;
  }

  set name(name: string | null) {
    this.#name = name;
    this.#updateDisplayName();
  }

  remove() {
    if (this.#engine) this.#engine.player.removePlayer(this.id);
  }

  async #updateDisplayName() {
    let newDisplayName = "";

    if (this.handle) newDisplayName = this.handle;
    else if (this.name) newDisplayName = this.name;
    else newDisplayName = `Guest ${toHex(this.id)}`;

    this.#displayName = newDisplayName;

    if (this.#engine) {
      const player = this.#engine.player.getPlayer(this.id);
      if (player) player.name = newDisplayName;
    }
  }
}

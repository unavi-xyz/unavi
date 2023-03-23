import { Engine } from "engine";

export class Player {
  readonly id: number;

  #name: string | null = null;
  #address: string | null = null;
  #avatar: string | null = null;
  #grounded = false;

  #engine: Engine | null = null;

  constructor(id: number) {
    this.id = id;
  }

  get name() {
    return this.#name;
  }

  set name(name: string | null) {
    this.#name = name;

    if (this.#engine) {
      const player = this.#engine.player.getPlayer(this.id);
      if (player) player.name = name;
    }
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

  get address() {
    return this.#address;
  }

  set address(address: string | null) {
    this.#address = address;
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

  get engine() {
    return this.#engine;
  }

  set engine(engine: Engine | null) {
    this.#engine = engine;
    if (!engine) return;

    const player = engine.player.getPlayer(this.id) || engine.player.addPlayer(this.id);

    player.name = this.#name;
    player.avatar = this.#avatar;
    player.grounded = this.#grounded;
  }

  remove() {
    if (this.#engine) this.#engine.player.removePlayer(this.id);
  }
}

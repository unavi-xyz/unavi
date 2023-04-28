import { Engine } from "engine";
import { providers, Signer } from "ethers";

import { toHex } from "../utils/toHex";

/**
 * Used to store information about a player as it comes in from the server.
 * Once the engine is provided, the player will be added to the engine.
 */
export class Player {
  readonly id: number;

  #address: string | null = null;
  #avatar: string | null = null;
  #engine: Engine | null = null;
  #ethersProvider: providers.Provider | Signer | null = null;
  #grounded = false;
  #name: string | null = null;

  #displayName = "";

  constructor(id: number) {
    this.id = id;
  }

  get address() {
    return this.#address;
  }

  set address(address: string | null) {
    this.#address = address;
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

  get ethersProvider() {
    return this.#ethersProvider;
  }

  set ethersProvider(ethersProvider: providers.Provider | Signer | null) {
    this.#ethersProvider = ethersProvider;
    this.#updateDisplayName();
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

    // if (this.address && this.ethersProvider) {
    //   const profileId = await fetchDefaultProfileId(this.address, this.ethersProvider);

    //   if (profileId !== null) {
    //     const handle = await fetchProfileHandle(profileId, this.ethersProvider);

    //     if (handle) newDisplayName = handle.string;
    //     else if (!this.name) newDisplayName = this.address.substring(0, 6);
    //   }
    // }

    if (!newDisplayName && this.name) newDisplayName = this.name;
    if (!newDisplayName) newDisplayName = `Guest ${toHex(this.id)}`;

    if (newDisplayName === this.#displayName) return;

    this.#displayName = newDisplayName;

    if (this.#engine) {
      const player = this.#engine.player.getPlayer(this.id);
      if (player) player.name = newDisplayName;
    }
  }
}

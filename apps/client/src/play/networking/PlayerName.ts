import { Engine } from "engine";
import { nanoid } from "nanoid";

import { getProfileByAddress } from "../../../app/api/profiles/address/[address]/helper";
import { usePlayStore } from "../../../app/play/[id]/store";
import { toHex } from "../../utils/toHex";
import { addChatMessage } from "../utils/addChatMessage";

export class PlayerName {
  readonly id: number;
  #engine: Engine;

  #displayName = "Guest";
  #nickname: string | null = null;
  #address: string | null = null;

  constructor(id: number, engine: Engine) {
    this.id = id;
    this.#engine = engine;
    this.displayName = `Guest ${this.hexId}`;
  }

  get hexId() {
    return toHex(this.id);
  }

  get displayName() {
    return this.#displayName;
  }

  set displayName(value: string) {
    if (this.#displayName === value) return;
    this.#displayName = value;

    const player = this.#engine.player.getPlayer(this.id);
    if (player) player.name = value;

    const { playerId } = usePlayStore.getState();
    if (this.id === playerId) console.info("🪪 You are now", value);
    else console.info("🪪 Player", this.hexId, "is now", value);
  }

  get nickname() {
    return this.#nickname;
  }

  set nickname(nickname: string | null) {
    this.#nickname = nickname;
    this.#updateDisplayName();
  }

  get address() {
    return this.#address;
  }

  set address(address: string | null) {
    this.#address = address;
    this.#updateDisplayName();
  }

  async #updateDisplayName() {
    if (this.address) {
      const profile = await getProfileByAddress(this.address);
      if (profile?.handle) {
        this.displayName = profile.handle.string;
        return;
      } else if (!this.nickname) {
        this.displayName = this.address.substring(0, 6);
        return;
      }
    }

    if (this.nickname) {
      this.displayName = this.nickname;
      return;
    }

    this.displayName = `Guest ${this.hexId}`;
  }

  chat(text: string) {
    addChatMessage({
      type: "chat",
      id: nanoid(),
      playerId: this.id,
      displayName: this.displayName,
      text,
      timestamp: Date.now(),
    });
  }
}

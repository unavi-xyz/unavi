import { nanoid } from "nanoid";

import { TrpcContext } from "../../client/trpc";
import { numberToHexDisplay } from "../../utils/numberToHexDisplay";
import { addChatMessage } from "../utils/addChatMessage";

export class PlayerName {
  readonly id: number;
  #trpc: TrpcContext;

  #displayName: string;
  #nickname: string | null = null;
  #address: string | null = null;

  constructor(id: number, trpc: TrpcContext) {
    this.id = id;
    this.#trpc = trpc;
    this.#displayName = `Guest ${this.hexId}`;
  }

  get hexId() {
    return numberToHexDisplay(this.id);
  }

  get displayName() {
    return this.#displayName;
  }

  set displayName(value: string) {
    if (this.#displayName === value) return;
    this.#displayName = value;
    console.info("🪪 Player", this.hexId, "is now", value);
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
      const profile = await this.#trpc.social.profile.byAddress.fetch({ address: this.address });
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

  sendChatMessage(text: string, timestamp = Date.now()) {
    addChatMessage({
      type: "chat",
      id: nanoid(),
      playerId: this.id,
      displayName: this.displayName,
      text,
      timestamp,
    });
  }
}

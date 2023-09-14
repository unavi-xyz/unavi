import { NodeItem } from "./NodeItem";

export class SceneTree {
  readonly items = new Map<bigint, NodeItem>();

  #rootId?: bigint;

  root?: NodeItem;

  get rootId() {
    return this.#rootId;
  }

  set rootId(value: bigint | undefined) {
    if (this.#rootId === value) return;

    this.#rootId = value;

    if (value) {
      this.root = this.items.get(value);
    } else {
      this.root = undefined;
    }
  }
}

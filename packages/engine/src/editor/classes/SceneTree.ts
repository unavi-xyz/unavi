import { TreeItem } from "./TreeItem";

export class SceneTree {
  readonly items = new Map<bigint, TreeItem>();

  #rootId?: bigint;

  root?: TreeItem;

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

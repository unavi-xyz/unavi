import {
  EditNode_Collider_Type,
  EditNode_RigidBody_Type,
} from "@unavi/protocol";

import { editorStore } from "../store";

export class TreeItem {
  readonly id: bigint;

  name = "";
  translation: [number, number, number] = [0, 0, 0];
  rotation: [number, number, number, number] = [0, 0, 0, 1];
  scale: [number, number, number] = [1, 1, 1];

  locked = false;

  colliderType?: EditNode_Collider_Type;
  rigidBodyType?: EditNode_RigidBody_Type;

  collider = {
    height: 1,
    meshId: 0n,
    radius: 1,
    size: [1, 1, 1] as [number, number, number],
  };

  #parentId?: bigint;
  #childrenIds: bigint[] = [];

  constructor(id: bigint) {
    this.id = id;
  }

  get parentId(): bigint | undefined {
    return this.#parentId;
  }

  set parentId(value: bigint | undefined) {
    if (this.#parentId === value) return;
    if (this.#parentId) {
      const parent = editorStore.get(editorStore.items).get(this.#parentId);
      this.#parentId = undefined;

      if (parent) {
        parent.removeChild(this);
      }
    }

    if (value) {
      const parent = editorStore.get(editorStore.items).get(value);
      if (parent) {
        this.#parentId = value;
        parent.addChild(this);
      }
    }
  }

  get childrenIds(): bigint[] {
    return this.#childrenIds;
  }

  get children(): TreeItem[] {
    return this.childrenIds.map((id) => {
      const item = editorStore.get(editorStore.items).get(id);
      if (!item) throw new Error(`Item with id ${id} not found`);
      return item;
    });
  }

  get parent(): TreeItem | undefined {
    if (!this.#parentId) return undefined;
    return editorStore.get(editorStore.items).get(this.#parentId);
  }

  addChild(child: TreeItem) {
    if (child.parentId !== this.id) {
      child.parentId = this.id;
    }

    this.childrenIds.push(child.id);
  }

  removeChild(child: TreeItem) {
    child.parentId = undefined;

    const index = this.childrenIds.indexOf(child.id);
    if (index !== -1) this.childrenIds.splice(index, 1);
  }

  clearChildren() {
    for (const child of this.children) {
      child.parentId = undefined;
    }

    this.childrenIds.length = 0;
  }

  sortChildren() {
    this.childrenIds.sort((a, b) => {
      const aItem = editorStore.get(editorStore.items).get(a);
      const bItem = editorStore.get(editorStore.items).get(b);

      if (aItem?.childrenIds?.length && !bItem?.childrenIds?.length) {
        return 1;
      } else if (!aItem?.childrenIds?.length && bItem?.childrenIds?.length) {
        return -1;
      } else {
        return aItem?.name?.localeCompare(bItem?.name || "") || 0;
      }
    });
  }

  destroy() {
    this.parent?.removeChild(this);
    this.clearChildren();

    const items = editorStore.get(editorStore.items);
    items.delete(this.id);
  }
}

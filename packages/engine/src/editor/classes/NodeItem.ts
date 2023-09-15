import {
  EditNode_Collider_Type,
  EditNode_RigidBody_Type,
} from "@unavi/protocol";

import { useSceneStore } from "../sceneStore";

export class NodeItem {
  readonly id: string;
  readonly entityId: bigint;

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

  constructor(id: string, entityId: bigint) {
    this.id = id;
    this.entityId = entityId;
  }

  get parentId(): bigint | undefined {
    return this.#parentId;
  }

  set parentId(value: bigint | undefined) {
    if (this.#parentId === value) return;

    if (this.#parentId) {
      const parent = useSceneStore.getState().items.get(this.#parentId);
      this.#parentId = undefined;

      if (parent) {
        parent.removeChild(this);
      }
    }

    if (value) {
      const parent = useSceneStore.getState().items.get(value);
      if (parent) {
        this.#parentId = value;
        parent.addChild(this);
      }
    }
  }

  get childrenIds(): bigint[] {
    return this.#childrenIds;
  }

  get children(): NodeItem[] {
    return this.childrenIds.map((id) => {
      const item = useSceneStore.getState().items.get(id);
      if (!item) throw new Error(`Item with id ${id} not found`);
      return item;
    });
  }

  get parent(): NodeItem | undefined {
    if (!this.#parentId) return undefined;
    return useSceneStore.getState().items.get(this.#parentId);
  }

  addChild(child: NodeItem) {
    if (child.parentId !== this.entityId) {
      child.parentId = this.entityId;
    }

    this.#childrenIds = [...this.#childrenIds, child.entityId];
  }

  removeChild(child: NodeItem) {
    child.parentId = undefined;

    const index = this.childrenIds.indexOf(child.entityId);
    if (index !== -1) {
      this.#childrenIds = [...this.#childrenIds].splice(index, 1);
    }
  }

  clearChildren() {
    for (const child of this.children) {
      child.parentId = undefined;
    }

    this.#childrenIds = [];
  }

  sortChildren() {
    const items = useSceneStore.getState().items;

    this.#childrenIds = [...this.#childrenIds].sort((a, b) => {
      const aItem = items.get(a);
      const bItem = items.get(b);

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

    const items = useSceneStore.getState().items;
    items.delete(this.entityId);
    useSceneStore.setState({ items: new Map(items) });
  }
}

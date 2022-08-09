import { nanoid } from "nanoid";

export class TreeItem {
  readonly id = nanoid();

  name = "";
  threeUUID: string | null = null;

  parent: TreeItem | null = null;
  children: TreeItem[] = [];

  constructor() {}

  addChild(child: TreeItem) {
    child.removeFromParent();
    this.children.push(child);
    child.parent = this;
  }

  removeFromParent() {
    if (this.parent) {
      this.parent.children = this.parent.children.filter((child) => child !== this);
      this.parent = null;
    }
  }
}

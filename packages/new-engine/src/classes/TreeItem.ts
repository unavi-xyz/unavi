import { nanoid } from "nanoid";

export class TreeItem {
  readonly id = nanoid();

  name = "";
  threeUUID: string | null = null;

  parent: TreeItem | null = null;
  children: TreeItem[] = [];

  constructor(json?: TreeItemJSON) {
    if (!json) return;

    this.id = json.id;
    this.name = json.name;
    this.threeUUID = json.threeUUID;
    json.children.forEach((child) => this.addChild(new TreeItem(child)));
  }

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

  toJSON(): TreeItemJSON {
    return {
      id: this.id,
      name: this.name,
      threeUUID: this.threeUUID,
      children: this.children.map((child) => child.toJSON()),
    };
  }
}

export type TreeItemJSON = {
  id: string;
  name: string;
  threeUUID: string | null;
  children: TreeItemJSON[];
};

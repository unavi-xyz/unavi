import { nanoid } from "nanoid";
import { OBJ_NAMES } from "..";

export class SceneObject {
  id: string;
  name: OBJ_NAMES;
  component: JSX.Element;

  constructor(name: OBJ_NAMES, component: JSX.Element) {
    this.id = nanoid();
    this.name = name;
    this.component = component;
  }

  clone() {
    return new SceneObject(this.name, this.component);
  }
}

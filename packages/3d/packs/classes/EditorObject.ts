import { MutableRefObject } from "react";
import { Triplet } from "@react-three/cannon";

import { SceneObject } from "./SceneObject";

export class EditorObject {
  id: string;

  params: SceneObject = new SceneObject();
  ref: MutableRefObject<any> | null = null;

  constructor(params?: SceneObject) {
    if (params) this.params = params;
    this.id = this.params.id;
  }

  load() {
    if (!this.ref?.current) return;

    this.ref.current.position.set(...this.params.position);
    this.ref.current.rotation.set(...this.params.rotation);
    this.ref.current.scale.set(...this.params.scale);
  }

  save() {
    if (!this.ref?.current) return;

    this.params.position = this.ref.current.position.toArray();
    this.params.scale = this.ref.current.scale.toArray();
    this.params.rotation = this.ref.current.rotation
      .toArray()
      .slice(0, 3) as Triplet;
  }

  clone() {
    return new EditorObject(this.params);
  }
}

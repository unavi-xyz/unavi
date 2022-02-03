import { MutableRefObject } from "react";
import { Triplet } from "@react-three/cannon";
import { nanoid } from "nanoid";

import { Params } from "./Params";

//each SceneObject represents an object within the scene
export class SceneObject {
  id: string;

  ref: MutableRefObject<any> | null = null;

  params: Params = new Params();

  constructor(params?: Params) {
    this.id = nanoid();
    if (params) this.params = params;
  }

  load() {
    if (!this.ref?.current) return;

    console.log("👨‍👧", this.params);

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
    return new SceneObject(this.params);
  }
}

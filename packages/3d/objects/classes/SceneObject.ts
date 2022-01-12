import { MutableRefObject } from "react";
import { Triplet } from "@react-three/cannon";
import { Object3D } from "three";
import { nanoid } from "nanoid";

import { OBJ_NAMES } from "../..";

export class SceneObject {
  id: string;
  name: OBJ_NAMES;

  ref: MutableRefObject<Object3D> | null;

  position: Triplet;
  rotation: Triplet;
  scale: Triplet;

  constructor(
    name: OBJ_NAMES,
    position: Triplet = [0, 0, 0],
    rotation: Triplet = [0, 0, 0],
    scale: Triplet = [1, 1, 1]
  ) {
    this.id = nanoid();
    this.name = name;

    this.ref = null;

    this.position = position;
    this.rotation = rotation;
    this.scale = scale;
  }

  save() {
    if (!this.ref) return;

    this.position = this.ref.current.position.toArray();
    this.rotation = this.ref.current.rotation.toArray().slice(0, 3) as Triplet;
    this.scale = this.ref.current.scale.toArray();
  }

  load() {
    if (!this.ref) return;

    this.ref.current.position.fromArray(this.position);
    this.ref.current.rotation.fromArray(this.rotation);
    this.ref.current.scale.fromArray(this.scale);
  }

  clone() {
    return new SceneObject(this.name, this.position, this.rotation, this.scale);
  }
}

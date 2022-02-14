import { MutableRefObject } from "react";

import { ASSETS, ASSET_NAMES } from "../assets";
import { PARAM_NAMES } from "./Asset";
import { SceneObject } from "./SceneObject";

export class EditorObject<T extends ASSET_NAMES> {
  id: string;
  instance: SceneObject<T>;

  ref: MutableRefObject<any> | undefined = undefined;

  constructor(type: ASSET_NAMES, params: typeof ASSETS[T]["params"]) {
    this.instance = new SceneObject<T>(type, params);
    this.id = this.instance.id;
  }

  load() {
    if (!this.ref?.current) return;

    if (PARAM_NAMES.position in this.instance.params) {
      this.ref.current.position.set(...this.instance.params.position);
    }

    if (PARAM_NAMES.rotation in this.instance.params) {
      this.ref.current.rotation.set(...this.instance.params.rotation);
    }

    if (PARAM_NAMES.scale in this.instance.params) {
      this.ref.current.scale.set(...this.instance.params.scale);
    }
  }

  save() {
    if (!this.ref?.current) return;

    if (PARAM_NAMES.position in this.instance.params) {
      this.instance.params.position = this.ref.current.position.toArray();
    }

    if (PARAM_NAMES.rotation in this.instance.params) {
      this.instance.params.rotation = this.ref.current.rotation
        .toArray()
        .slice(0, 3);
    }

    if (PARAM_NAMES.scale in this.instance.params) {
      this.instance.params.scale = this.ref.current.scale.toArray();
    }
  }

  clone() {
    return new EditorObject(this.instance.type, { ...this.instance.params });
  }
}

import { Group, Object3D, ObjectLoader, Quaternion, Vector3 } from "three";

import { ToRenderMessage } from "../types";
import { disposeTree } from "../utils/disposeTree";
import { RenderWorker } from "./RenderWorker";

export class Tree {
  tree: Object3D = new Group();

  #worker: RenderWorker;
  #loader = new ObjectLoader();

  constructor(worker: RenderWorker) {
    this.#worker = worker;
    this.#worker.scene.add(this.tree);
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { id, subject, data } = event.data;

    switch (subject) {
      case "set_tree":
        this.#setTree(data.json);
        break;
      case "add_object":
        this.#addObject(data.json, data.parent);
        break;
      case "remove_object":
        this.#removeObject(data.uuid);
        break;
      case "move_object":
        this.#moveObject(data.uuid, data.parent);
      case "get_object":
        const object = this.#getObject(data.uuid);
        const json = object?.toJSON();
        this.#worker.postMessage({ id, subject: "got_object", data: { json } });
        break;
    }
  }

  #setTree(json: any) {
    this.tree.removeFromParent();
    disposeTree(this.tree);

    const object = this.#loader.parse(json);

    this.#worker.scene.add(object);
    this.tree = object;
  }

  #addObject(json: any, parent = "root") {
    const object = this.#loader.parse(json);
    const parentObject = this.#getObject(parent);
    if (parentObject) parentObject.add(object);
  }

  #removeObject(uuid: string) {
    const object = this.#getObject(uuid);
    if (!object) return;

    object.removeFromParent();
    disposeTree(object);
  }

  #moveObject(uuid: string, parent: string) {
    const object = this.#getObject(uuid);
    if (!object) return;

    const parentObject = this.#getObject(parent);
    if (!parentObject) return;

    // Save object transform
    const position = object.getWorldPosition(new Vector3());
    const rotation = object.getWorldQuaternion(new Quaternion());

    // Add object to new parent
    parentObject.add(object);

    // Restore object transform
    const inverseParentRotation = parentObject.getWorldQuaternion(new Quaternion()).invert();
    object.position.copy(parentObject.worldToLocal(position));
    object.quaternion.multiplyQuaternions(rotation, inverseParentRotation);
  }

  #getObject(uuid: string) {
    if (uuid === "root") return this.tree;
    const object = this.tree.getObjectByProperty("uuid", uuid);
    return object;
  }
}

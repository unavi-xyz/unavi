import { PerspectiveCamera, Scene } from "three";

import { FakePointerEvent } from "../classes/OrbitControls";
import { SceneLink } from "../classes/SceneLink";
import { TransformControls } from "../classes/TransformControls";
import { PluginState } from "../RenderWorker";
import { ToRenderMessage } from "../types";
import { Plugin } from "./Plugin";

export class TransformControlsPlugin extends Plugin {
  #target = new EventTarget();
  #sceneLink: SceneLink;
  #transformControls: TransformControls;

  constructor(
    camera: PerspectiveCamera,
    sceneLink: SceneLink,
    scene: Scene,
    state: PluginState
  ) {
    super();

    this.#sceneLink = sceneLink;
    this.#transformControls = new TransformControls(camera, this.#target);
    scene.add(this.#transformControls);

    this.#transformControls.addEventListener("mouseDown", () => {
      state.usingTransformControls = true;
    });

    this.#transformControls.addEventListener("mouseUp", () => {
      state.usingTransformControls = false;

      // Send new transform to main thread
      const object = this.#transformControls.object;
      if (!object) throw new Error("No object found");
      const id = this.#sceneLink.findId(object);
      if (id === undefined) throw new Error("Object id not found");

      this.#sceneLink.saveTransform(id);
    });
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { subject, data } = event.data;

    switch (subject) {
      case "set_transform_target":
        if (data === null) this.#transformControls.detach();
        else {
          const object = this.#sceneLink.findObject(data);
          if (object) this.#transformControls.attach(object);
          else throw new Error(`Object not found: ${data}`);
        }
        break;
      case "set_transform_mode":
        this.#transformControls.mode = data;
        break;
      case "pointermove":
        const pointerMoveEvent: FakePointerEvent = new CustomEvent(
          "pointermove",
          { detail: data }
        );
        this.#target.dispatchEvent(pointerMoveEvent);
        break;
      case "pointerdown":
        const pointerDownEvent: FakePointerEvent = new CustomEvent(
          "pointerdown",
          { detail: data }
        );
        this.#target.dispatchEvent(pointerDownEvent);
        break;
      case "pointerup":
        const pointerUpEvent: FakePointerEvent = new CustomEvent("pointerup", {
          detail: data,
        });
        this.#target.dispatchEvent(pointerUpEvent);
        break;
      case "remove_entity":
        const attachedObject = this.#transformControls.object;
        if (attachedObject) {
          const id = this.#sceneLink.findId(attachedObject);
          if (id === undefined) throw new Error("Object id not found");
          // Detach if attached object is removed
          if (id === data.entityId) this.#transformControls.detach();
        }
    }
  }

  destroy() {
    this.#transformControls.detach();
    this.#transformControls.dispose();
  }
}

import { PerspectiveCamera, Scene } from "three";

import { PluginState } from "../../RenderWorker";
import { SceneLoader } from "../../SceneLoader/SceneLoader";
import { ToRenderMessage } from "../../types";
import { FakePointerEvent } from "../types";
import { TransformControls } from "./TransformControls";

export class TransformControlsPlugin {
  #target = new EventTarget();
  #sceneLoader: SceneLoader;
  #transformControls: TransformControls;

  constructor(
    camera: PerspectiveCamera,
    sceneLoader: SceneLoader,
    scene: Scene,
    state: PluginState
  ) {
    this.#sceneLoader = sceneLoader;
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

      const id = this.#sceneLoader.findId(object);
      if (id === undefined) throw new Error("Object id not found");

      this.#sceneLoader.saveTransform(id);
    });
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { subject, data } = event.data;

    switch (subject) {
      case "set_transform_target": {
        if (data === null) this.#transformControls.detach();
        else {
          const object = this.#sceneLoader.findObject(data);
          if (object) this.#transformControls.attach(object);
          else throw new Error(`Object not found: ${data}`);
        }
        break;
      }

      case "set_transform_mode": {
        this.#transformControls.mode = data;
        break;
      }

      case "pointermove": {
        const pointerMoveEvent: FakePointerEvent = new CustomEvent(
          "pointermove",
          { detail: data }
        );
        this.#target.dispatchEvent(pointerMoveEvent);
        break;
      }

      case "pointerdown": {
        const pointerDownEvent: FakePointerEvent = new CustomEvent(
          "pointerdown",
          { detail: data }
        );
        this.#target.dispatchEvent(pointerDownEvent);
        break;
      }

      case "pointerup": {
        const pointerUpEvent: FakePointerEvent = new CustomEvent("pointerup", {
          detail: data,
        });
        this.#target.dispatchEvent(pointerUpEvent);
        break;
      }

      case "remove_node": {
        const attachedObject = this.#transformControls.object;
        if (attachedObject) {
          const id = this.#sceneLoader.findId(attachedObject);
          if (id === undefined) throw new Error("Object id not found");
          // Detach if attached object is removed
          if (id === data.nodeId) this.#transformControls.detach();
        }
        break;
      }
    }
  }

  destroy() {
    this.#transformControls.detach();
    this.#transformControls.dispose();
  }
}

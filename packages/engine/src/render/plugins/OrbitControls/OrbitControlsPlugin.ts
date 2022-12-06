import { PerspectiveCamera } from "three";

import { PluginState } from "../../RenderWorker";
import { ToRenderMessage } from "../../types";
import { FakePointerEvent, FakeWheelEvent, RenderPlugin } from "../types";
import { OrbitControls } from "./OrbitControls";

export class OrbitControlsPlugin implements RenderPlugin {
  #target = new EventTarget();
  #orbitControls: OrbitControls;
  #state: PluginState;

  constructor(
    camera: PerspectiveCamera,
    canvasWidth: number,
    canvasHeight: number,
    state: PluginState
  ) {
    this.#state = state;
    this.#orbitControls = new OrbitControls(
      camera,
      this.#target,
      canvasWidth,
      canvasHeight
    );
    this.#orbitControls.enableDamping = true;
    this.#orbitControls.dampingFactor = 0.05;

    camera.position.set(-1, 2, 5);
    camera.lookAt(0, 0, 0);
  }

  animate() {
    this.#orbitControls.update();
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { subject, data } = event.data;

    switch (subject) {
      case "pointermove": {
        const pointerMoveEvent: FakePointerEvent = new CustomEvent(
          "pointermove",
          {
            detail: data,
          }
        );
        this.#target.dispatchEvent(pointerMoveEvent);
        break;
      }

      case "pointerdown": {
        if (this.#state.usingTransformControls) return;
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

      case "pointercancel": {
        const pointerCancelEvent: FakePointerEvent = new CustomEvent(
          "pointercancel",
          {
            detail: data,
          }
        );
        this.#target.dispatchEvent(pointerCancelEvent);
        break;
      }

      case "wheel": {
        const wheelEvent: FakeWheelEvent = new CustomEvent("wheel", {
          detail: data,
        });
        this.#target.dispatchEvent(wheelEvent);
        break;
      }
    }
  }

  destroy() {
    this.#orbitControls.dispose();
  }
}

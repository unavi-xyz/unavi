import { PerspectiveCamera } from "three";

import { InputMessage } from "../../input/messages";
import { ThreeOrbitControls } from "./ThreeOrbitControls";

export class OrbitControls {
  #target = new EventTarget();
  #orbitControls: ThreeOrbitControls;

  constructor(camera: PerspectiveCamera) {
    this.#orbitControls = new ThreeOrbitControls(camera, this.#target);
    this.#orbitControls.enableDamping = true;
    this.#orbitControls.dampingFactor = 0.05;

    camera.position.set(-1, 2, 5);
    camera.lookAt(0, 0, 0);
  }

  setSize(width: number, height: number) {
    this.#orbitControls.canvasSize.set(width, height);
  }

  update() {
    this.#orbitControls.update();
  }

  onmessage({ subject, data }: InputMessage) {
    switch (subject) {
      case "pointermove": {
        const pointerMoveEvent = new CustomEvent("pointermove", { detail: data });
        this.#target.dispatchEvent(pointerMoveEvent);
        break;
      }

      case "pointerdown": {
        // if (this.#state.usingTransformControls) return;

        const pointerDownEvent = new CustomEvent("pointerdown", { detail: data });
        this.#target.dispatchEvent(pointerDownEvent);
        break;
      }

      case "pointerup": {
        const pointerUpEvent = new CustomEvent("pointerup", { detail: data });
        this.#target.dispatchEvent(pointerUpEvent);
        break;
      }

      case "pointercancel": {
        const pointerCancelEvent = new CustomEvent("pointercancel", { detail: data });
        this.#target.dispatchEvent(pointerCancelEvent);
        break;
      }

      case "wheel": {
        const wheelEvent = new CustomEvent("wheel", { detail: data });
        this.#target.dispatchEvent(wheelEvent);
        break;
      }
    }
  }

  destroy() {
    this.#orbitControls.dispose();
  }
}

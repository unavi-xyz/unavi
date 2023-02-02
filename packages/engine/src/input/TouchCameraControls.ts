import { INPUT_ARRAY_ROUNDING } from "../constants";
import { InputModule } from "./InputModule";

export class TouchCameraControls {
  #module: InputModule;

  touchId: number | undefined = undefined;

  #angle: number | undefined = undefined;
  #fixedX: number | undefined = undefined;
  #fixedY: number | undefined = undefined;
  #innerX: number | undefined = undefined;
  #innerY: number | undefined = undefined;
  #cameraX = 0;
  #cameraY = 0;
  #prevCameraX = 0;
  #prevCameraY = 0;

  constructor(module: InputModule) {
    this.#module = module;
  }

  onTouchStart(x: number, y: number) {
    this.#fixedX = x;
    this.#fixedY = y;
    this.#innerX = x;
    this.#innerY = y;

    this.#cameraX = this.#prevCameraX;
    this.#cameraY = this.#prevCameraY;
  }

  onTouchMove(x: number, y: number) {
    if (this.#fixedX === undefined || this.#fixedY === undefined) return;

    this.#innerX = x;
    this.#innerY = y;

    this.#angle = Math.atan2(this.#innerY - this.#fixedY, this.#innerX - this.#fixedX);
  }

  onTouchEnd() {
    this.touchId = undefined;
    this.#angle = undefined;
    this.#fixedX = undefined;
    this.#fixedY = undefined;
    this.#innerX = undefined;
    this.#innerY = undefined;
  }

  update() {
    if (
      this.#angle === undefined ||
      this.#fixedX === undefined ||
      this.#fixedY === undefined ||
      this.#innerX === undefined ||
      this.#innerY === undefined
    )
      return;

    const x =
      (this.#fixedX - this.#innerX) / this.#module.engine.overlayCanvas.width + this.#cameraX;
    const y =
      (this.#fixedY - this.#innerY) / this.#module.engine.overlayCanvas.height + this.#cameraY;

    this.#prevCameraX = x;
    this.#prevCameraY = y;

    // Write to rotation input
    if (this.#module.engine.inputRotation) {
      Atomics.store(this.#module.engine.inputRotation, 0, x * INPUT_ARRAY_ROUNDING);
      Atomics.store(this.#module.engine.inputRotation, 1, y * INPUT_ARRAY_ROUNDING);
    }
  }
}

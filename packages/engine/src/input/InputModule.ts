import { Engine } from "../Engine";
import { KeyboardInput } from "./KeyboardInput";
import { MobileInput } from "./MobileInput";

/**
 * Handles input from the user.
 *
 * @group Modules
 */
export class InputModule {
  readonly engine: Engine;

  keyboard: KeyboardInput;
  mobile: MobileInput;

  #isLocked = false;
  #canvas: HTMLCanvasElement | null = null;

  constructor(engine: Engine) {
    this.engine = engine;
    this.keyboard = new KeyboardInput(this);
    this.mobile = new MobileInput(this);

    document.addEventListener("pointerlockchange", this.#onPointerLockChange);
  }

  get canvas() {
    return this.#canvas;
  }

  set canvas(value: HTMLCanvasElement | null) {
    if (value === this.#canvas) return;
    this.#canvas = value;
    this.keyboard.canvas = value;
    this.mobile.canvas = value;
  }

  get isLocked() {
    return this.#isLocked;
  }

  set isLocked(value: boolean) {
    if (value === this.#isLocked) return;
    this.#isLocked = value;

    if (value) {
      this.mobile.stop();
    } else {
      this.mobile.start();
    }
  }

  #onPointerLockChange = () => {
    this.isLocked = document.pointerLockElement === this.engine.overlayCanvas;
  };

  destroy() {
    this.keyboard.destroy();
    this.mobile.destroy();
  }
}

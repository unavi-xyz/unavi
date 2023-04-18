import { InputModule } from "./InputModule";
import { Joystick } from "./Joystick";
import { TouchCameraControls } from "./TouchCameraControls";

/**
 * Handles touch input from the user.
 * Uses a joystick for movement on the left side of the screen,
 * and drag controls for camera movement on the right side of the screen.
 */
export class MobileInput {
  joystick: Joystick;
  touch: TouchCameraControls;

  #animationFrame: number | undefined = undefined;
  #canvas: HTMLCanvasElement | null = null;

  constructor(module: InputModule) {
    this.joystick = new Joystick(module);
    this.touch = new TouchCameraControls(module);

    this.start();
  }

  get canvas() {
    return this.#canvas;
  }

  set canvas(value: HTMLCanvasElement | null) {
    if (value === this.#canvas) return;

    if (this.#canvas) {
      this.#canvas.removeEventListener("touchstart", this.#onTouchStart);
      this.#canvas.removeEventListener("touchmove", this.#onTouchMove);
      this.#canvas.removeEventListener("touchend", this.#onTouchEnd);
    }

    if (value) {
      value.addEventListener("touchstart", this.#onTouchStart.bind(this));
      value.addEventListener("touchmove", this.#onTouchMove.bind(this));
      value.addEventListener("touchend", this.#onTouchEnd.bind(this));
    }

    this.#canvas = value;
    this.joystick.canvas = value;
    this.touch.canvas = value;
  }

  #onTouchStart(event: TouchEvent) {
    const joystickId = this.joystick.touchId;
    const touchId = this.touch.touchId;

    const touch = Array.from(event.touches).find((touch) => {
      return touch.identifier !== joystickId && touch.identifier !== touchId;
    });
    if (!touch) return;

    event.preventDefault();

    // If left side of canvas, use joystick
    if (this.canvas) {
      if (touch.clientX < this.canvas.width / 2) {
        this.joystick.touchId = touch.identifier;
        this.joystick.onTouchStart(touch.clientX, touch.clientY);
      } else {
        this.touch.touchId = touch.identifier;
        this.touch.onTouchStart(touch.clientX, touch.clientY);
      }
    }
  }

  #onTouchMove(event: TouchEvent) {
    const joystickId = this.joystick.touchId;
    const touchId = this.touch.touchId;

    const joystickTouch = Array.from(event.touches).find(
      (touch) => touch.identifier === joystickId
    );
    const touchTouch = Array.from(event.touches).find((touch) => touch.identifier === touchId);

    if (joystickTouch !== undefined)
      this.joystick.onTouchMove(joystickTouch.clientX, joystickTouch.clientY);
    if (touchTouch !== undefined) this.touch.onTouchMove(touchTouch.clientX, touchTouch.clientY);
  }

  #onTouchEnd(event: TouchEvent) {
    const joystickId = this.joystick.touchId;
    const touchId = this.touch.touchId;

    const joystickTouch = Array.from(event.touches).find(
      (touch) => touch.identifier === joystickId
    );
    const touchTouch = Array.from(event.touches).find((touch) => touch.identifier === touchId);

    if (joystickTouch === undefined) this.joystick.onTouchEnd();
    if (touchTouch === undefined) this.touch.onTouchEnd();
  }

  start() {
    this.stop();
    this.#animationFrame = requestAnimationFrame(this.update.bind(this));
  }

  stop() {
    if (this.#animationFrame !== undefined) cancelAnimationFrame(this.#animationFrame);
  }

  update() {
    this.#animationFrame = requestAnimationFrame(this.update.bind(this));

    this.joystick.update();
    this.touch.update();
  }

  destroy() {
    this.stop();

    this.canvas = null;
  }
}

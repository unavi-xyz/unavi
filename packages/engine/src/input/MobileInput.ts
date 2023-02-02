import { InputModule } from "./InputModule";
import { Joystick } from "./Joystick";
import { TouchCameraControls } from "./TouchCameraControls";

export class MobileInput {
  #module: InputModule;

  joystick: Joystick;
  touch: TouchCameraControls;

  #animationFrame: number | undefined = undefined;

  constructor(module: InputModule) {
    this.#module = module;
    this.joystick = new Joystick(module);
    this.touch = new TouchCameraControls(module);

    const canvas = module.engine.overlayCanvas;
    canvas.addEventListener("touchstart", this.#onTouchStart.bind(this));
    canvas.addEventListener("touchmove", this.#onTouchMove.bind(this));
    canvas.addEventListener("touchend", this.#onTouchEnd.bind(this));

    this.start();
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
    if (touch.clientX < this.#module.engine.overlayCanvas.width / 2) {
      this.joystick.touchId = touch.identifier;
      this.joystick.onTouchStart(touch.clientX, touch.clientY);
    } else {
      this.touch.touchId = touch.identifier;
      this.touch.onTouchStart(touch.clientX, touch.clientY);
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

    const canvas = this.#module.engine.overlayCanvas;
    canvas.removeEventListener("touchstart", this.#onTouchStart.bind(this));
    canvas.removeEventListener("touchmove", this.#onTouchMove.bind(this));
    canvas.removeEventListener("touchend", this.#onTouchEnd.bind(this));
  }
}

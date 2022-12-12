import { RenderThread } from "../render/RenderThread";

export class TouchCameraControls {
  #canvas: HTMLCanvasElement;
  #renderThread: RenderThread;

  #angle: number | undefined = undefined;
  #fixedX: number | undefined = undefined;
  #fixedY: number | undefined = undefined;
  #innerX: number | undefined = undefined;
  #innerY: number | undefined = undefined;
  #animationFrame: number | undefined = undefined;

  #cameraX = 0;
  #cameraY = 0;
  #prevCameraX = 0;
  #prevCameraY = 0;

  constructor(canvas: HTMLCanvasElement, renderThread: RenderThread) {
    this.#canvas = canvas;
    this.#renderThread = renderThread;

    canvas.addEventListener("touchstart", this.#onTouchStart.bind(this));
    canvas.addEventListener("touchmove", this.#onTouchMove.bind(this));
    canvas.addEventListener("touchend", this.#onTouchEnd.bind(this));
    canvas.addEventListener("touchcancel", this.#onTouchEnd.bind(this));

    this.update();
  }

  #onTouchStart(event: TouchEvent) {
    const touch = event.touches[0];
    if (!touch) return;

    // Only take input from the right side of the screen
    if (touch.clientX < this.#canvas.width / 2) return;

    this.#fixedX = touch.clientX;
    this.#fixedY = touch.clientY;
    this.#innerX = touch.clientX;
    this.#innerY = touch.clientY;

    this.#cameraX = this.#prevCameraX;
    this.#cameraY = this.#prevCameraY;
  }

  #onTouchMove(event: TouchEvent) {
    const touch = event.touches[0];
    if (!touch) return;

    if (this.#fixedX === undefined || this.#fixedY === undefined) return;

    this.#innerX = touch.clientX;
    this.#innerY = touch.clientY;

    this.#angle = Math.atan2(this.#innerY - this.#fixedY, this.#innerX - this.#fixedX);
  }

  #onTouchEnd() {
    this.#fixedX = undefined;
    this.#fixedY = undefined;
    this.#innerX = undefined;
    this.#innerY = undefined;
    this.#angle = undefined;
  }

  update() {
    this.#animationFrame = requestAnimationFrame(this.update.bind(this));

    if (
      this.#angle === undefined ||
      this.#fixedX === undefined ||
      this.#fixedY === undefined ||
      this.#innerX === undefined ||
      this.#innerY === undefined
    )
      return;

    // Send camera input to render thread
    const x = (this.#fixedX - this.#innerX) / this.#canvas.width + this.#cameraX;
    const y = (this.#fixedY - this.#innerY) / this.#canvas.height + this.#cameraY;

    this.#prevCameraX = x;
    this.#prevCameraY = y;

    this.#renderThread.postMessage({ subject: "set_camera_input_vector", data: [x, y] });
  }

  destroy() {
    if (this.#animationFrame !== undefined) cancelAnimationFrame(this.#animationFrame);

    this.#canvas.removeEventListener("touchstart", this.#onTouchStart);
    this.#canvas.removeEventListener("touchmove", this.#onTouchMove);
    this.#canvas.removeEventListener("touchend", this.#onTouchEnd);
    this.#canvas.removeEventListener("touchcancel", this.#onTouchEnd);
  }
}

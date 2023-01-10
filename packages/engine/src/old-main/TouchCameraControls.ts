import { RenderThread } from "../render/RenderThread";

export class TouchCameraControls {
  #canvas: HTMLCanvasElement;
  #renderThread: RenderThread;

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

  constructor(canvas: HTMLCanvasElement, renderThread: RenderThread) {
    this.#canvas = canvas;
    this.#renderThread = renderThread;

    this.update();
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

    // Send camera input to render thread
    const x = (this.#fixedX - this.#innerX) / this.#canvas.width + this.#cameraX;
    const y = (this.#fixedY - this.#innerY) / this.#canvas.height + this.#cameraY;

    this.#prevCameraX = x;
    this.#prevCameraY = y;

    this.#renderThread.postMessage({ subject: "set_camera_input_vector", data: [x, y] });
  }
}

import { RenderThread } from "../render/RenderThread";

const FIXED_RADIUS = 70;
const INNER_RADIUS = 30;

export class Joystick {
  #canvas: HTMLCanvasElement;
  #renderThread: RenderThread;

  touchId: number | undefined = undefined;

  #ctx: CanvasRenderingContext2D;
  #angle: number | undefined = undefined;
  #fixedX: number | undefined = undefined;
  #fixedY: number | undefined = undefined;
  #innerX: number | undefined = undefined;
  #innerY: number | undefined = undefined;

  constructor(canvas: HTMLCanvasElement, renderThread: RenderThread) {
    this.#canvas = canvas;
    this.#renderThread = renderThread;

    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("Could not get 2d context from canvas");

    this.#ctx = ctx;
  }

  onTouchStart(x: number, y: number) {
    this.#fixedX = x;
    this.#fixedY = y;
    this.#innerX = x;
    this.#innerY = y;
  }

  onTouchMove(x: number, y: number) {
    if (this.#fixedX === undefined || this.#fixedY === undefined) return;

    this.#innerX = x;
    this.#innerY = y;

    this.#angle = Math.atan2(this.#innerY - this.#fixedY, this.#innerX - this.#fixedX);

    // If inner circle is outside joystick radius, reduce it to the circumference
    if (
      !((this.#innerX - this.#fixedX) ** 2 + (this.#innerY - this.#fixedY) ** 2 < FIXED_RADIUS ** 2)
    ) {
      this.#innerX = Math.round(Math.cos(this.#angle) * FIXED_RADIUS + this.#fixedX);
      this.#innerY = Math.round(Math.sin(this.#angle) * FIXED_RADIUS + this.#fixedY);
    }
  }

  onTouchEnd() {
    this.touchId = undefined;
    this.#angle = undefined;
    this.#fixedX = undefined;
    this.#fixedY = undefined;
    this.#innerX = undefined;
    this.#innerY = undefined;

    this.#renderThread.setPlayerInputVector([0, 0]);
  }

  update() {
    this.#ctx.clearRect(0, 0, this.#canvas.width, this.#canvas.height);

    if (
      this.#angle === undefined ||
      this.#fixedX === undefined ||
      this.#fixedY === undefined ||
      this.#innerX === undefined ||
      this.#innerY === undefined
    )
      return;

    // Draw joystick outer circle
    this.#ctx.beginPath();
    this.#ctx.fillStyle = "#0004";
    this.#ctx.arc(this.#fixedX, this.#fixedY, FIXED_RADIUS, 0, 2 * Math.PI);
    this.#ctx.fill();
    this.#ctx.closePath();

    // Draw inner circle
    this.#ctx.beginPath();
    this.#ctx.fillStyle = "#0009";
    this.#ctx.arc(this.#innerX, this.#innerY, INNER_RADIUS, 0, 2 * Math.PI);
    this.#ctx.fill();
    this.#ctx.closePath();

    // Send movement input to render thread
    const x = -Math.max(-1, Math.min(1, (this.#innerX - this.#fixedX) / FIXED_RADIUS));
    const y = -Math.max(-1, Math.min(1, (this.#innerY - this.#fixedY) / FIXED_RADIUS));
    this.#renderThread.setPlayerInputVector([x, y]);
  }
}

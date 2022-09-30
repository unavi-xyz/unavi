import { RenderThread } from "../render/RenderThread";
import { PhysicsThread } from "./PhysicsThread";

export class Player {
  #canvas: HTMLCanvasElement;
  #renderThread: RenderThread;
  #physicsThread: PhysicsThread;

  #click = this.#onClick.bind(this);
  #keydown = this.#onKeyDown.bind(this);
  #keyup = this.#onKeyUp.bind(this);
  #pointerlockchange = this.#onPointerlockChange.bind(this);
  #mousemove = this.#onMouseMove.bind(this);

  #pressingW = false;
  #pressingS = false;
  #pressingA = false;
  #pressingD = false;

  #isLocked = false;

  constructor(
    canvas: HTMLCanvasElement,
    renderThread: RenderThread,
    physicsThread: PhysicsThread
  ) {
    this.#canvas = canvas;
    this.#renderThread = renderThread;
    this.#physicsThread = physicsThread;

    this.#canvas.addEventListener("click", this.#click);
    document.addEventListener("keydown", this.#keydown);
    document.addEventListener("keyup", this.#keyup);
    document.addEventListener("pointerlockchange", this.#pointerlockchange);
    document.addEventListener("mousemove", this.#mousemove);
  }

  destroy() {
    this.#canvas.removeEventListener("click", this.#click);
    document.removeEventListener("keydown", this.#keydown);
    document.removeEventListener("keyup", this.#keyup);
    document.removeEventListener("pointerlockchange", this.#pointerlockchange);
    document.removeEventListener("mousemove", this.#mousemove);

    document.exitPointerLock();
  }

  #onClick() {
    this.#canvas.requestPointerLock();
  }

  #onPointerlockChange() {
    this.#isLocked = document.pointerLockElement === this.#canvas;
  }

  #onMouseMove(event: MouseEvent) {
    if (!this.#isLocked) return;
    const { movementX, movementY } = event;
    this.#renderThread.mouseMove(movementX, movementY);
  }

  #onKeyDown(event: KeyboardEvent) {
    switch (event.key) {
      case "w":
        this.#pressingW = true;
        break;
      case "s":
        this.#pressingS = true;
        break;
      case "a":
        this.#pressingA = true;
        break;
      case "d":
        this.#pressingD = true;
        break;
      case " ":
        this.#physicsThread.jump();
        break;
      default:
        return;
    }

    this.#updateVelocity();
  }

  #onKeyUp(event: KeyboardEvent) {
    switch (event.key) {
      case "w":
        this.#pressingW = false;
        break;
      case "s":
        this.#pressingS = false;
        break;
      case "a":
        this.#pressingA = false;
        break;
      case "d":
        this.#pressingD = false;
        break;
      default:
        return;
    }

    this.#updateVelocity();
  }

  #updateVelocity() {
    const wForce = this.#pressingW ? 1 : 0;
    const sForce = this.#pressingS ? 1 : 0;
    const aForce = this.#pressingA ? 1 : 0;
    const dForce = this.#pressingD ? 1 : 0;
    const forward = wForce - sForce;
    const right = aForce - dForce;
    const direction: [number, number] = [right, forward];

    // Send direction to render thread
    this.#renderThread.setPlayerInputVector(direction);
  }
}

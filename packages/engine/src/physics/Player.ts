import { RenderThread } from "../render/RenderThread";
import { PhysicsThread } from "./PhysicsThread";

/*
 * Handles user input for the player.
 */
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

  #jumpInterval: NodeJS.Timeout | null = null;

  #isLocked = false;

  constructor(
    canvas: HTMLCanvasElement,
    renderThread: RenderThread,
    physicsThread: PhysicsThread
  ) {
    this.#canvas = canvas;
    this.#renderThread = renderThread;
    this.#physicsThread = physicsThread;

    this.#canvas.addEventListener("click", this.#click.bind(this));
    document.addEventListener("keydown", this.#keydown.bind(this));
    document.addEventListener("keyup", this.#keyup.bind(this));
    document.addEventListener(
      "pointerlockchange",
      this.#pointerlockchange.bind(this)
    );
    document.addEventListener("mousemove", this.#mousemove.bind(this));
    document.addEventListener("blur", this.#onBlur.bind(this));
  }

  destroy() {
    this.#canvas.removeEventListener("click", this.#click);
    document.removeEventListener("keydown", this.#keydown);
    document.removeEventListener("keyup", this.#keyup);
    document.removeEventListener("pointerlockchange", this.#pointerlockchange);
    document.removeEventListener("mousemove", this.#mousemove);
    document.removeEventListener("blur", this.#onBlur);

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
      case "w": {
        this.#pressingW = true;
        this.#updateVelocity();
        break;
      }

      case "s": {
        this.#pressingS = true;
        this.#updateVelocity();
        break;
      }

      case "a": {
        this.#pressingA = true;
        this.#updateVelocity();
        break;
      }

      case "d": {
        this.#pressingD = true;
        this.#updateVelocity();
        break;
      }

      case " ": {
        if (!this.#jumpInterval) {
          // Jump now
          this.#physicsThread.jump();

          // Send a jump command on an interval while the spacebar is held down
          this.#jumpInterval = setInterval(() => {
            this.#physicsThread.jump();
          }, 200);
        }
        break;
      }
    }
  }

  #onKeyUp(event: KeyboardEvent) {
    switch (event.key) {
      case "w": {
        this.#pressingW = false;
        this.#updateVelocity();
        break;
      }

      case "s": {
        this.#pressingS = false;
        this.#updateVelocity();
        break;
      }

      case "a": {
        this.#pressingA = false;
        this.#updateVelocity();
        break;
      }

      case "d": {
        this.#pressingD = false;
        this.#updateVelocity();
        break;
      }

      case " ": {
        if (this.#jumpInterval !== null) {
          clearInterval(this.#jumpInterval);
          this.#jumpInterval = null;
        }
        break;
      }
    }
  }

  #onBlur() {
    this.#pressingW = false;
    this.#pressingS = false;
    this.#pressingA = false;
    this.#pressingD = false;
    this.#updateVelocity();

    if (this.#jumpInterval !== null) {
      clearInterval(this.#jumpInterval);
      this.#jumpInterval = null;
    }
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

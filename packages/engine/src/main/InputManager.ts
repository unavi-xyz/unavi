import { PhysicsThread } from "../physics/PhysicsThread";
import { RenderThread } from "../render/RenderThread";
import { getPointerData } from "./getPointerData";
import { Joystick } from "./Joystick";
import { TouchCameraControls } from "./TouchCameraControls";

export class InputManager {
  canvas = document.createElement("canvas");

  #cameraType: "orbit" | "player";
  #renderThread: RenderThread;
  #physicsThread: PhysicsThread;
  #joystick: Joystick | null = null;
  #touchControls: TouchCameraControls | null = null;

  #pressingW = false;
  #pressingS = false;
  #pressingA = false;
  #pressingD = false;
  #isLocked = false;

  #jumpInterval: NodeJS.Timeout | null = null;
  #animationFrame: number | null = null;

  constructor(
    renderThread: RenderThread,
    physicsThread: PhysicsThread,
    cameraType: "orbit" | "player"
  ) {
    this.#renderThread = renderThread;
    this.#physicsThread = physicsThread;
    this.#cameraType = cameraType;

    if (cameraType === "player") {
      this.#joystick = new Joystick(this.canvas, renderThread);
      this.#touchControls = new TouchCameraControls(this.canvas, renderThread);
    }

    this.canvas.addEventListener("click", this.#onClick.bind(this));
    this.canvas.addEventListener("contextmenu", this.#onContextMenu.bind(this));
    this.canvas.addEventListener("wheel", this.#onWheel.bind(this));

    this.canvas.addEventListener("pointermove", this.#onPointerMove.bind(this));
    this.canvas.addEventListener("pointerup", this.#onPointerUp.bind(this));
    this.canvas.addEventListener("pointerdown", this.#onPointerDown.bind(this));
    this.canvas.addEventListener("pointercancel", this.#onPointerCancel.bind(this));

    this.canvas.addEventListener("touchstart", this.#onTouchStart.bind(this));
    this.canvas.addEventListener("touchmove", this.#onTouchMove.bind(this));
    this.canvas.addEventListener("touchend", this.#onTouchEnd.bind(this));
    this.canvas.addEventListener("touchcancel", this.#onTouchEnd.bind(this));

    document.addEventListener("keydown", this.#onKeyDown.bind(this));
    document.addEventListener("keyup", this.#onKeyUp.bind(this));
    document.addEventListener("mousemove", this.#onMouseMove.bind(this));
    document.addEventListener("pointerlockchange", this.#onPointerLockChange.bind(this));
    document.addEventListener("pointerlockchange", this.#onPointerLockChange.bind(this));

    this.update();
  }

  update() {
    this.#animationFrame = requestAnimationFrame(this.update.bind(this));

    this.#joystick?.update();
    this.#touchControls?.update();
  }

  destroy() {
    this.canvas.removeEventListener("click", this.#onClick);
    this.canvas.removeEventListener("contextmenu", this.#onContextMenu);
    this.canvas.removeEventListener("wheel", this.#onWheel);

    this.canvas.removeEventListener("pointermove", this.#onPointerMove);
    this.canvas.removeEventListener("pointerup", this.#onPointerUp);
    this.canvas.removeEventListener("pointerdown", this.#onPointerDown);
    this.canvas.removeEventListener("pointercancel", this.#onPointerCancel);

    this.canvas.removeEventListener("touchstart", this.#onTouchStart);
    this.canvas.removeEventListener("touchmove", this.#onTouchMove);
    this.canvas.removeEventListener("touchend", this.#onTouchEnd);
    this.canvas.removeEventListener("touchcancel", this.#onTouchEnd);

    document.removeEventListener("keydown", this.#onKeyDown);
    document.removeEventListener("keyup", this.#onKeyUp);
    document.removeEventListener("mousemove", this.#onMouseMove);
    document.removeEventListener("pointerlockchange", this.#onPointerLockChange);
    document.removeEventListener("pointerlockchange", this.#onPointerLockChange);

    document.exitPointerLock();
  }

  #onTouchStart(event: TouchEvent) {
    if (!this.#joystick || !this.#touchControls) return;

    const joystickId = this.#joystick.touchId;
    const touchControlsId = this.#touchControls.touchId;

    const touch = Array.from(event.touches).find((touch) => {
      return touch.identifier !== joystickId && touch.identifier !== touchControlsId;
    });
    if (!touch) return;

    event.preventDefault();

    // If left side of canvas, use joystick
    if (touch.clientX < this.canvas.width / 2) {
      this.#joystick.touchId = touch.identifier;
      this.#joystick.onTouchStart(touch.clientX, touch.clientY);
    } else {
      this.#touchControls.touchId = touch.identifier;
      this.#touchControls.onTouchStart(touch.clientX, touch.clientY);
    }
  }

  #onTouchMove(event: TouchEvent) {
    if (!this.#joystick || !this.#touchControls) return;

    const joystickId = this.#joystick.touchId;
    const touchControlsId = this.#touchControls.touchId;

    const joystickTouch = Array.from(event.touches).find(
      (touch) => touch.identifier === joystickId
    );
    const touchControlsTouch = Array.from(event.touches).find(
      (touch) => touch.identifier === touchControlsId
    );

    if (joystickTouch !== undefined)
      this.#joystick.onTouchMove(joystickTouch.clientX, joystickTouch.clientY);
    if (touchControlsTouch !== undefined)
      this.#touchControls.onTouchMove(touchControlsTouch.clientX, touchControlsTouch.clientY);
  }

  #onTouchEnd(event: TouchEvent) {
    if (!this.#joystick || !this.#touchControls) return;

    const joystickId = this.#joystick.touchId;
    const touchControlsId = this.#touchControls.touchId;

    const joystickTouch = Array.from(event.touches).find(
      (touch) => touch.identifier === joystickId
    );
    const touchControlsTouch = Array.from(event.touches).find(
      (touch) => touch.identifier === touchControlsId
    );

    if (joystickTouch === undefined) this.#joystick.onTouchEnd();
    if (touchControlsTouch === undefined) this.#touchControls.onTouchEnd();
  }

  #onClick() {
    if (this.#cameraType === "player") this.canvas.requestPointerLock();
  }

  #onMouseMove(event: MouseEvent) {
    if (!this.#isLocked) return;
    this.#renderThread.mouseMove(event.movementX, event.movementY);
  }

  #onContextMenu(event: Event) {
    event.preventDefault();
  }

  #onPointerMove(event: PointerEvent) {
    this.#renderThread.postMessage({
      subject: "pointermove",
      data: getPointerData(event, this.canvas),
    });
  }

  #onPointerUp(event: PointerEvent) {
    this.canvas.releasePointerCapture(event.pointerId);

    this.#renderThread.postMessage({
      subject: "pointerup",
      data: getPointerData(event, this.canvas),
    });
  }

  #onPointerDown(event: PointerEvent) {
    const isPointerLocked = document.pointerLockElement === this.canvas;
    if (isPointerLocked) return;

    this.canvas.setPointerCapture(event.pointerId);

    this.#renderThread.postMessage({
      subject: "pointerdown",
      data: getPointerData(event, this.canvas),
    });
  }

  #onPointerCancel(event: PointerEvent) {
    this.#renderThread.postMessage({
      subject: "pointercancel",
      data: getPointerData(event, this.canvas),
    });
  }

  #onWheel(event: WheelEvent) {
    event.preventDefault();
    this.#renderThread.postMessage({
      subject: "wheel",
      data: {
        deltaY: event.deltaY,
      },
    });
  }

  #onKeyDown(event: KeyboardEvent) {
    const key = event.key.toLowerCase();

    if (event.shiftKey) this.#physicsThread.setSprinting(true);

    switch (key) {
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
    const key = event.key.toLowerCase();

    if (!event.shiftKey) this.#physicsThread.setSprinting(false);

    switch (key) {
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

  #onPointerLockChange() {
    this.#isLocked = document.pointerLockElement === this.canvas;
    if (this.#isLocked) return;

    this.#pressingW = false;
    this.#pressingS = false;
    this.#pressingA = false;
    this.#pressingD = false;
    this.#updateVelocity();

    this.#physicsThread.setSprinting(false);

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

import { INPUT_ARRAY_ROUNDING } from "../constants";
import { ControlsType } from "../types";
import { InputModule } from "./InputModule";
import { PointerData } from "./messages";

/**
 * Handles keyboard and mouse input.
 */
export class KeyboardInput {
  #module: InputModule;
  #canvas: HTMLCanvasElement | null = null;

  controls: ControlsType = "player";
  isLocked = false;

  keys = {
    a: false,
    d: false,
    down: false,
    left: false,
    right: false,
    s: false,
    up: false,
    w: false,
  };

  #jumpInterval: NodeJS.Timeout | null = null;
  #capturedPointerId: number | null = null;

  constructor(module: InputModule) {
    this.#module = module;

    document.addEventListener("keydown", this.#onKeyDown);
    document.addEventListener("keyup", this.#onKeyUp);
    document.addEventListener("mousemove", this.#onMouseMove);
    document.addEventListener("pointerlockchange", this.#onPointerLockChange);
  }

  get canvas() {
    return this.#canvas;
  }

  set canvas(value: HTMLCanvasElement | null) {
    if (value === this.#canvas) return;

    if (this.#capturedPointerId !== null)
      this.#canvas?.releasePointerCapture(this.#capturedPointerId);
    this.#capturedPointerId = null;

    if (this.#canvas) {
      this.#canvas.removeEventListener("pointermove", this.#onPointerMove);
      this.#canvas.removeEventListener("pointerup", this.#onPointerUp);
      this.#canvas.removeEventListener("pointerdown", this.#onPointerDown);
      this.#canvas.removeEventListener("pointercancel", this.#onPointerCancel);

      this.#canvas.removeEventListener("click", this.#onClick);
      this.#canvas.removeEventListener("contextmenu", this.#onContextMenu);
      this.#canvas.removeEventListener("wheel", this.#onWheel);
    }

    if (value) {
      value.addEventListener("pointermove", this.#onPointerMove);
      value.addEventListener("pointerup", this.#onPointerUp);
      value.addEventListener("pointerdown", this.#onPointerDown);
      value.addEventListener("pointercancel", this.#onPointerCancel);

      value.addEventListener("click", this.#onClick);
      value.addEventListener("contextmenu", this.#onContextMenu);
      value.addEventListener("wheel", this.#onWheel);
    }

    this.#canvas = value;
  }

  destroy() {
    this.canvas = null;

    document.removeEventListener("keydown", this.#onKeyDown);
    document.removeEventListener("keyup", this.#onKeyUp);
    document.removeEventListener("mousemove", this.#onMouseMove);
    document.removeEventListener("pointerlockchange", this.#onPointerLockChange);
  }

  #onClick = () => {
    if (!this.canvas) return;
    if (this.controls === "player") this.canvas.requestPointerLock();
  };

  #onContextMenu = (event: Event) => {
    event.preventDefault();
  };

  #onMouseMove = (event: MouseEvent) => {
    if (!this.isLocked) return;

    this.#module.engine.render.send({
      data: { x: event.movementX, y: event.movementY },
      subject: "mousemove",
    });
  };

  #onWheel = (event: WheelEvent) => {
    event.preventDefault();

    this.#module.engine.render.send({
      data: { deltaY: event.deltaY },
      subject: "wheel",
    });
  };

  #onKeyDown = (event: KeyboardEvent) => {
    const key = event.key.toLowerCase();

    if (event.shiftKey) {
      this.#module.engine.render.send({ data: true, subject: "set_sprinting" });
      this.#module.engine.physics.send({ data: true, subject: "set_sprinting" });
    }

    switch (key) {
      case "w": {
        this.keys.w = true;
        break;
      }

      case "s": {
        this.keys.s = true;
        break;
      }

      case "a": {
        this.keys.a = true;
        break;
      }

      case "d": {
        this.keys.d = true;
        break;
      }

      case "arrowup": {
        this.keys.up = true;
        break;
      }

      case "arrowleft": {
        this.keys.left = true;
        break;
      }

      case "arrowdown": {
        this.keys.down = true;
        break;
      }

      case "arrowright": {
        this.keys.right = true;
        break;
      }

      case " ": {
        if (!this.#jumpInterval) {
          // Jump now
          this.#module.engine.physics.send({ data: null, subject: "jump" });

          // Send a jump command on an interval while the spacebar is held down
          this.#jumpInterval = setInterval(() => {
            this.#module.engine.physics.send({ data: null, subject: "jump" });
          }, 100);
        }
        break;
      }
    }

    this.#updateVelocity();
  };

  #onKeyUp = (event: KeyboardEvent) => {
    const key = event.key.toLowerCase();

    if (!event.shiftKey) {
      this.#module.engine.render.send({ data: false, subject: "set_sprinting" });
      this.#module.engine.physics.send({ data: false, subject: "set_sprinting" });
    }

    switch (key) {
      case "w": {
        this.keys.w = false;
        break;
      }

      case "s": {
        this.keys.s = false;
        break;
      }

      case "a": {
        this.keys.a = false;
        break;
      }

      case "d": {
        this.keys.d = false;
        break;
      }

      case "arrowup": {
        this.keys.up = false;
        break;
      }

      case "arrowleft": {
        this.keys.left = false;
        break;
      }

      case "arrowdown": {
        this.keys.down = false;
        break;
      }

      case "arrowright": {
        this.keys.right = false;
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

    this.#updateVelocity();
  };

  #onPointerMove = (event: PointerEvent) => {
    if (!this.canvas) return;
    this.#module.engine.render.send({
      data: getPointerData(event, this.canvas),
      subject: "pointermove",
    });
  };

  #onPointerUp = (event: PointerEvent) => {
    this.#capturedPointerId = null;

    if (!this.canvas) return;
    this.canvas.releasePointerCapture(event.pointerId);

    this.#module.engine.render.send({
      data: getPointerData(event, this.canvas),
      subject: "pointerup",
    });
  };

  #onPointerDown = (event: PointerEvent) => {
    if (!this.canvas) return;

    const isPointerLocked = document.pointerLockElement === this.canvas;
    if (isPointerLocked) return;

    this.#capturedPointerId = event.pointerId;
    this.canvas.setPointerCapture(event.pointerId);

    this.#module.engine.render.send({
      data: getPointerData(event, this.canvas),
      subject: "pointerdown",
    });
  };

  #onPointerCancel = (event: PointerEvent) => {
    if (!this.canvas) return;

    this.#capturedPointerId = null;
    this.canvas.releasePointerCapture(event.pointerId);

    this.#module.engine.render.send({
      data: getPointerData(event, this.canvas),
      subject: "pointercancel",
    });
  };

  #onPointerLockChange = () => {
    if (!this.canvas) return;

    this.isLocked = document.pointerLockElement === this.canvas;
    if (this.isLocked) return;

    // Unpress all keys on exit pointer lock
    Object.keys(this.keys).forEach((key) => {
      this.keys[key as keyof typeof this.keys] = false;
    });

    this.#updateVelocity();

    this.#module.engine.render.send({ data: false, subject: "set_sprinting" });
    this.#module.engine.physics.send({ data: false, subject: "set_sprinting" });

    if (this.#jumpInterval !== null) {
      clearInterval(this.#jumpInterval);
      this.#jumpInterval = null;
    }
  };

  #updateVelocity() {
    const forward = Number(this.keys.w || this.keys.up);
    const back = Number(this.keys.s || this.keys.down);
    const left = Number(this.keys.a || this.keys.left);
    const right = Number(this.keys.d || this.keys.right);

    const x = right - left;
    const y = forward - back;

    const length = Math.sqrt(x * x + y * y);
    const normalX = x === 0 ? 0 : x / length;
    const normalY = y === 0 ? 0 : y / length;

    // Write to position input
    if (this.#module.engine.inputPosition) {
      Atomics.store(this.#module.engine.inputPosition, 0, normalX * INPUT_ARRAY_ROUNDING);
      Atomics.store(this.#module.engine.inputPosition, 1, normalY * INPUT_ARRAY_ROUNDING);
    }
  }
}

function getPointerData(event: PointerEvent, canvas: HTMLCanvasElement): PointerData {
  let pointer;

  if (canvas.ownerDocument.pointerLockElement) {
    pointer = { button: event.button, x: 0, y: 0 };
  } else {
    const rect = canvas.getBoundingClientRect();
    pointer = {
      button: event.button,
      x: ((event.clientX - rect.left) / rect.width) * 2 - 1,
      y: (-(event.clientY - rect.top) / rect.height) * 2 + 1,
    };
  }

  return {
    button: event.button,
    clientX: event.clientX,
    clientY: event.clientY,
    ctrlKey: event.ctrlKey,
    metaKey: event.metaKey,
    pageX: event.pageX,
    pageY: event.pageY,
    pointer,
    pointerId: event.pointerId,
    pointerType: event.pointerType,
    shiftKey: event.shiftKey,
  };
}

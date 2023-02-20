import { INPUT_ARRAY_ROUNDING } from "../constants";
import { ControlsType } from "../Engine";
import { InputModule } from "./InputModule";
import { PointerData } from "./messages";

/**
 * Handles keyboard and mouse input.
 */
export class KeyboardInput {
  #module: InputModule;

  controls: ControlsType = "player";
  isLocked = false;

  keys = {
    w: false,
    a: false,
    s: false,
    d: false,
    up: false,
    left: false,
    down: false,
    right: false,
  };

  #jumpInterval: NodeJS.Timeout | null = null;
  #capturedPointerId: number | null = null;

  constructor(module: InputModule) {
    this.#module = module;
    const canvas = module.engine.overlayCanvas;

    canvas.addEventListener("click", this.#onClick);
    canvas.addEventListener("contextmenu", this.#onContextMenu);
    canvas.addEventListener("wheel", this.#onWheel);

    canvas.addEventListener("pointermove", this.#onPointerMove);
    canvas.addEventListener("pointerup", this.#onPointerUp);
    canvas.addEventListener("pointerdown", this.#onPointerDown);
    canvas.addEventListener("pointercancel", this.#onPointerCancel);

    document.addEventListener("keydown", this.#onKeyDown);
    document.addEventListener("keyup", this.#onKeyUp);
    document.addEventListener("mousemove", this.#onMouseMove);
    document.addEventListener("pointerlockchange", this.#onPointerLockChange);
  }

  destroy() {
    const canvas = this.#module.engine.overlayCanvas;

    if (this.#capturedPointerId !== null) canvas.releasePointerCapture(this.#capturedPointerId);

    canvas.removeEventListener("click", this.#onClick);
    canvas.removeEventListener("contextmenu", this.#onContextMenu);
    canvas.removeEventListener("wheel", this.#onWheel);

    canvas.removeEventListener("pointermove", this.#onPointerMove);
    canvas.removeEventListener("pointerup", this.#onPointerUp);
    canvas.removeEventListener("pointerdown", this.#onPointerDown);
    canvas.removeEventListener("pointercancel", this.#onPointerCancel);

    document.removeEventListener("keydown", this.#onKeyDown);
    document.removeEventListener("keyup", this.#onKeyUp);
    document.removeEventListener("mousemove", this.#onMouseMove);
    document.removeEventListener("pointerlockchange", this.#onPointerLockChange);
  }

  getPointerData(event: PointerEvent): PointerData {
    const canvas = this.#module.engine.overlayCanvas;
    let pointer;

    if (canvas.ownerDocument.pointerLockElement) {
      pointer = { x: 0, y: 0, button: event.button };
    } else {
      const rect = canvas.getBoundingClientRect();
      pointer = {
        x: ((event.clientX - rect.left) / rect.width) * 2 - 1,
        y: (-(event.clientY - rect.top) / rect.height) * 2 + 1,
        button: event.button,
      };
    }

    return {
      pointerId: event.pointerId,
      pointerType: event.pointerType,
      clientX: event.clientX,
      clientY: event.clientY,
      pageX: event.pageX,
      pageY: event.pageY,
      button: event.button,
      ctrlKey: event.ctrlKey,
      shiftKey: event.shiftKey,
      metaKey: event.metaKey,
      pointer,
    };
  }

  #onClick = () => {
    if (this.controls === "player") this.#module.engine.overlayCanvas.requestPointerLock();
  };

  #onContextMenu = (event: Event) => {
    event.preventDefault();
  };

  #onMouseMove = (event: MouseEvent) => {
    if (!this.isLocked) return;

    this.#module.engine.render.send({
      subject: "mousemove",
      data: { x: event.movementX, y: event.movementY },
    });
  };

  #onWheel = (event: WheelEvent) => {
    event.preventDefault();

    this.#module.engine.render.send({
      subject: "wheel",
      data: { deltaY: event.deltaY },
    });
  };

  #onKeyDown = (event: KeyboardEvent) => {
    const key = event.key.toLowerCase();

    if (event.shiftKey) {
      this.#module.engine.render.send({ subject: "set_sprinting", data: true });
      this.#module.engine.physics.send({ subject: "set_sprinting", data: true });
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
          this.#module.engine.physics.send({ subject: "jump", data: null });

          // Send a jump command on an interval while the spacebar is held down
          this.#jumpInterval = setInterval(() => {
            this.#module.engine.physics.send({ subject: "jump", data: null });
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
      this.#module.engine.render.send({ subject: "set_sprinting", data: false });
      this.#module.engine.physics.send({ subject: "set_sprinting", data: false });
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
    this.#module.engine.render.send({
      subject: "pointermove",
      data: this.getPointerData(event),
    });
  };

  #onPointerUp = (event: PointerEvent) => {
    this.#capturedPointerId = null;
    this.#module.engine.overlayCanvas.releasePointerCapture(event.pointerId);

    this.#module.engine.render.send({
      subject: "pointerup",
      data: this.getPointerData(event),
    });
  };

  #onPointerDown = (event: PointerEvent) => {
    const isPointerLocked = document.pointerLockElement === this.#module.engine.overlayCanvas;
    if (isPointerLocked) return;

    this.#capturedPointerId = event.pointerId;
    this.#module.engine.overlayCanvas.setPointerCapture(event.pointerId);

    this.#module.engine.render.send({
      subject: "pointerdown",
      data: this.getPointerData(event),
    });
  };

  #onPointerCancel = (event: PointerEvent) => {
    this.#capturedPointerId = null;
    this.#module.engine.overlayCanvas.releasePointerCapture(event.pointerId);

    this.#module.engine.render.send({
      subject: "pointercancel",
      data: this.getPointerData(event),
    });
  };

  #onPointerLockChange = () => {
    this.isLocked = document.pointerLockElement === this.#module.engine.overlayCanvas;
    if (this.isLocked) return;

    // Unpress all keys on exit pointer lock
    Object.keys(this.keys).forEach((key) => {
      this.keys[key as keyof typeof this.keys] = false;
    });

    this.#updateVelocity();

    this.#module.engine.render.send({ subject: "set_sprinting", data: false });
    this.#module.engine.physics.send({ subject: "set_sprinting", data: false });

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

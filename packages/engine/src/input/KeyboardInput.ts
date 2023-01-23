import { INPUT_ARRAY_ROUNDING } from "../constants";
import { ControlsType } from "../Engine";
import { RenderModule } from "../render/RenderModule";
import { InputModule } from "./InputModule";
import { PointerData } from "./messages";

export class KeyboardInput {
  #module: InputModule;
  #canvas: HTMLCanvasElement;
  #render: RenderModule;

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

  constructor(module: InputModule) {
    this.#module = module;
    this.#canvas = module.canvas;
    this.#render = module.render;

    this.#canvas.addEventListener("click", this.#onClick);
    this.#canvas.addEventListener("contextmenu", this.#onContextMenu);
    this.#canvas.addEventListener("wheel", this.#onWheel);

    this.#canvas.addEventListener("pointermove", this.#onPointerMove);
    this.#canvas.addEventListener("pointerup", this.#onPointerUp);
    this.#canvas.addEventListener("pointerdown", this.#onPointerDown);
    this.#canvas.addEventListener("pointercancel", this.#onPointerCancel);

    document.addEventListener("keydown", this.#onKeyDown);
    document.addEventListener("keyup", this.#onKeyUp);
    document.addEventListener("mousemove", this.#onMouseMove);
    document.addEventListener("pointerlockchange", this.#onPointerLockChange);
  }

  destroy() {
    this.#canvas.removeEventListener("click", this.#onClick);
    this.#canvas.removeEventListener("contextmenu", this.#onContextMenu);
    this.#canvas.removeEventListener("wheel", this.#onWheel);

    this.#canvas.removeEventListener("pointermove", this.#onPointerMove);
    this.#canvas.removeEventListener("pointerup", this.#onPointerUp);
    this.#canvas.removeEventListener("pointerdown", this.#onPointerDown);
    this.#canvas.removeEventListener("pointercancel", this.#onPointerCancel);

    document.removeEventListener("keydown", this.#onKeyDown);
    document.removeEventListener("keyup", this.#onKeyUp);
    document.removeEventListener("mousemove", this.#onMouseMove);
    document.removeEventListener("pointerlockchange", this.#onPointerLockChange);
  }

  getPointerData(event: PointerEvent): PointerData {
    let pointer;

    if (this.#canvas.ownerDocument.pointerLockElement) {
      pointer = { x: 0, y: 0, button: event.button };
    } else {
      const rect = this.#canvas.getBoundingClientRect();
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
    if (this.controls === "player") this.#canvas.requestPointerLock();
  };

  #onContextMenu = (event: Event) => {
    event.preventDefault();
  };

  #onMouseMove = (event: MouseEvent) => {
    if (!this.isLocked) return;

    this.#render.toRenderThread({
      subject: "mousemove",
      data: { x: event.movementX, y: event.movementY },
    });
  };

  #onWheel = (event: WheelEvent) => {
    event.preventDefault();

    this.#render.toRenderThread({
      subject: "wheel",
      data: { deltaY: event.deltaY },
    });
  };

  #onKeyDown = (event: KeyboardEvent) => {
    const key = event.key.toLowerCase();

    // if (event.shiftKey) this.#physicsThread.setSprinting(true);

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
        // if (!this.#jumpInterval) {
        //   // Jump now
        //   this.#physicsThread.jump();

        //   // Send a jump command on an interval while the spacebar is held down
        //   this.#jumpInterval = setInterval(() => {
        //     this.#physicsThread.jump();
        //   }, 200);
        // }
        break;
      }
    }

    this.#updateVelocity();
  };

  #onKeyUp = (event: KeyboardEvent) => {
    const key = event.key.toLowerCase();

    // if (!event.shiftKey) this.#physicsThread.setSprinting(false);

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
        // if (this.#jumpInterval !== null) {
        //   clearInterval(this.#jumpInterval);
        //   this.#jumpInterval = null;
        // }
        break;
      }
    }

    this.#updateVelocity();
  };

  #onPointerMove = (event: PointerEvent) => {
    this.#render.toRenderThread({
      subject: "pointermove",
      data: this.getPointerData(event),
    });
  };

  #onPointerUp = (event: PointerEvent) => {
    this.#canvas.releasePointerCapture(event.pointerId);

    this.#render.toRenderThread({
      subject: "pointerup",
      data: this.getPointerData(event),
    });
  };

  #onPointerDown = (event: PointerEvent) => {
    const isPointerLocked = document.pointerLockElement === this.#canvas;
    if (isPointerLocked) return;

    this.#canvas.releasePointerCapture(event.pointerId);
    this.#canvas.setPointerCapture(event.pointerId);

    this.#render.toRenderThread({
      subject: "pointerdown",
      data: this.getPointerData(event),
    });
  };

  #onPointerCancel = (event: PointerEvent) => {
    this.#canvas.releasePointerCapture(event.pointerId);

    this.#render.toRenderThread({
      subject: "pointercancel",
      data: this.getPointerData(event),
    });
  };

  #onPointerLockChange = () => {
    this.isLocked = document.pointerLockElement === this.#canvas;
    if (this.isLocked) return;

    // Unpress all keys on exit pointer lock
    Object.keys(this.keys).forEach((key) => {
      this.keys[key as keyof typeof this.keys] = false;
    });

    this.#updateVelocity();

    // this.#physicsThread.setSprinting(false);

    // if (this.#jumpInterval !== null) {
    //   clearInterval(this.#jumpInterval);
    //   this.#jumpInterval = null;
    // }
  };

  #updateVelocity() {
    const forward = Number(this.keys.w || this.keys.up);
    const back = Number(this.keys.s || this.keys.down);
    const left = Number(this.keys.a || this.keys.left);
    const right = Number(this.keys.d || this.keys.right);

    const x = right - left;
    const y = forward - back;
    const input = { x, y };

    // Normalize direction
    const magnitude = Math.sqrt(input.x ** 2 + input.y ** 2);
    if (magnitude > 0) {
      input.x /= magnitude;
      input.y /= magnitude;
    }

    // Write to input array
    if (this.#module.inputArray) {
      Atomics.store(this.#module.inputArray, 0, input.x * INPUT_ARRAY_ROUNDING);
      Atomics.store(this.#module.inputArray, 1, input.y * INPUT_ARRAY_ROUNDING);
    }
  }
}

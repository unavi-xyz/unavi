import { RenderModule } from "../render/RenderModule";
import { Vec2 } from "../types";
import { PointerData } from "./messages";

export class KeyboardInput {
  #canvas: HTMLCanvasElement;
  #render: RenderModule;

  cameraType: "player" | "orbit" = "orbit";
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

  constructor(canvas: HTMLCanvasElement, render: RenderModule) {
    this.#canvas = canvas;
    this.#render = render;

    canvas.addEventListener("click", this.#onClick.bind(this));
    canvas.addEventListener("contextmenu", this.#onContextMenu.bind(this));
    canvas.addEventListener("wheel", this.#onWheel.bind(this));
    canvas.addEventListener("mousemove", this.#onMouseMove.bind(this));

    canvas.addEventListener("keydown", this.#onKeyDown.bind(this));
    canvas.addEventListener("keyup", this.#onKeyUp.bind(this));

    canvas.addEventListener("pointermove", this.#onPointerMove.bind(this));
    canvas.addEventListener("pointerup", this.#onPointerUp.bind(this));
    canvas.addEventListener("pointerdown", this.#onPointerDown.bind(this));
    canvas.addEventListener("pointercancel", this.#onPointerCancel.bind(this));

    canvas.addEventListener("pointerlockchange", this.#onPointerLockChange.bind(this));
    canvas.addEventListener("pointerlockchange", this.#onPointerLockChange.bind(this));
  }

  destroy() {
    const canvas = this.#canvas;

    canvas.removeEventListener("click", this.#onClick.bind(this));
    canvas.removeEventListener("contextmenu", this.#onContextMenu.bind(this));
    canvas.removeEventListener("mousemove", this.#onMouseMove.bind(this));
    canvas.removeEventListener("wheel", this.#onWheel.bind(this));

    canvas.removeEventListener("keydown", this.#onKeyDown.bind(this));
    canvas.removeEventListener("keyup", this.#onKeyUp.bind(this));

    canvas.removeEventListener("pointermove", this.#onPointerMove.bind(this));
    canvas.removeEventListener("pointerup", this.#onPointerUp.bind(this));
    canvas.removeEventListener("pointerdown", this.#onPointerDown.bind(this));
    canvas.removeEventListener("pointercancel", this.#onPointerCancel.bind(this));

    canvas.removeEventListener("pointerlockchange", this.#onPointerLockChange.bind(this));
    canvas.removeEventListener("pointerlockchange", this.#onPointerLockChange.bind(this));
  }

  getPointerData(event: PointerEvent): PointerData {
    let pointer;

    if (this.#canvas.ownerDocument.pointerLockElement) {
      pointer = {
        x: 0,
        y: 0,
        button: event.button,
      };
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

  #onClick() {
    if (this.cameraType === "player") this.#canvas.requestPointerLock();
  }

  #onContextMenu(event: Event) {
    event.preventDefault();
  }

  #onMouseMove(event: MouseEvent) {
    if (!this.isLocked) return;

    this.#render.toRenderThread({
      subject: "mousemove",
      data: { x: event.movementX, y: event.movementY },
    });
  }

  #onWheel(event: WheelEvent) {
    event.preventDefault();

    this.#render.toRenderThread({
      subject: "wheel",
      data: { deltaY: event.deltaY },
    });
  }

  #onKeyDown(event: KeyboardEvent) {
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
  }

  #onKeyUp(event: KeyboardEvent) {
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
  }

  #onPointerMove(event: PointerEvent) {
    this.#render.toRenderThread({
      subject: "pointermove",
      data: this.getPointerData(event),
    });
  }

  #onPointerUp(event: PointerEvent) {
    this.#canvas.releasePointerCapture(event.pointerId);

    this.#render.toRenderThread({
      subject: "pointerup",
      data: this.getPointerData(event),
    });
  }

  #onPointerDown(event: PointerEvent) {
    const isPointerLocked = document.pointerLockElement === this.#canvas;
    if (isPointerLocked) return;

    this.#canvas.setPointerCapture(event.pointerId);

    this.#render.toRenderThread({
      subject: "pointerdown",
      data: this.getPointerData(event),
    });
  }

  #onPointerCancel(event: PointerEvent) {
    this.#canvas.releasePointerCapture(event.pointerId);

    this.#render.toRenderThread({
      subject: "pointercancel",
      data: this.getPointerData(event),
    });
  }

  #onPointerLockChange() {
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
  }

  #updateVelocity() {
    const forward = Number(this.keys.w || this.keys.up);
    const back = Number(this.keys.s || this.keys.down);
    const left = Number(this.keys.a || this.keys.left);
    const right = Number(this.keys.d || this.keys.right);

    const x = left - right;
    const y = forward - back;
    const data: Vec2 = [x, y];

    // Normalize direction
    const magnitude = Math.sqrt(data[0] ** 2 + data[1] ** 2);
    if (magnitude > 0) {
      data[0] /= magnitude;
      data[1] /= magnitude;
    }

    // Send direction to render thread
    this.#render.toRenderThread({ subject: "player_input_direction", data });
  }
}

import { Vector2 } from "three";
import { PointerLockControls } from "three/examples/jsm/controls/PointerLockControls";

import { Engine } from "../Engine";

export class Player {
  #engine: Engine;
  #canvas: HTMLCanvasElement;
  #controls: PointerLockControls;
  #direction = new Vector2();

  #click = this.#onClick.bind(this);
  #keydown = this.#onKeyDown.bind(this);
  #keyup = this.#onKeyUp.bind(this);

  #pressingW = false;
  #pressingS = false;
  #pressingA = false;
  #pressingD = false;

  constructor(engine: Engine) {
    this.#engine = engine;
    this.#canvas = engine.renderManager.renderer.domElement;

    this.#controls = new PointerLockControls(engine.renderManager.camera, document.body);
    this.#controls.connect();

    this.#canvas.addEventListener("click", this.#click);
    document.addEventListener("keydown", this.#keydown);
    document.addEventListener("keyup", this.#keyup);

    this.#engine.gameThread.initPlayer();
  }

  destroy() {
    this.#canvas.removeEventListener("click", this.#click);
    document.removeEventListener("keydown", this.#keydown);
    document.removeEventListener("keyup", this.#keyup);

    this.#controls.disconnect();
    this.#controls.dispose();
  }

  #onClick() {
    this.#controls.lock();
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
        this.#engine.gameThread.jump();
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
    this.#direction.set(right, forward).normalize();

    // Send direction to render thread
    this.#engine.renderManager.setPlayerInputVector(this.#direction);
  }
}

import { BehaviorModule } from "./behavior/BehaviorModule";
import { DEFAULT_CONTROLS, DEFAULT_VISUALS } from "./constants";
import { InputModule } from "./input/InputModule";
import { PhysicsModule } from "./physics/PhysicsModule";
import { PlayerModules } from "./player/PlayerModule";
import { RenderModule } from "./render/RenderModule";
import { SceneModule } from "./scene/SceneModule";
import { ControlsType } from "./types";

export interface EngineOptions {
  canvas: HTMLCanvasElement;
  overlayCanvas: HTMLCanvasElement;
}

/**
 * The main engine class.
 *
 * Uses two canvases, one for rendering and one for UI elements.
 * Requires {@link https://web.dev/cross-origin-isolation-guide/ cross-origin isolation} to be enabled.
 */
export class Engine {
  readonly canvas: HTMLCanvasElement;
  readonly overlayCanvas: HTMLCanvasElement;

  readonly behavior: BehaviorModule;
  readonly input: InputModule;
  readonly physics: PhysicsModule;
  readonly player: PlayerModules;
  readonly render: RenderModule;
  readonly scene: SceneModule;

  inputPosition: Int16Array;
  inputRotation: Int16Array;
  userPosition: Int32Array;
  userRotation: Int16Array;
  cameraPosition: Int32Array;
  cameraYaw: Int16Array;

  #controls: ControlsType = DEFAULT_CONTROLS;
  #visuals = DEFAULT_VISUALS;

  /**
   * Creates a new engine instance.
   *
   * @param canvas The canvas to render to.
   * @param overlayCanvas The canvas to render the overlay to.
   */
  constructor({ canvas, overlayCanvas }: EngineOptions) {
    this.canvas = canvas;
    this.overlayCanvas = overlayCanvas;

    this.inputPosition = new Int16Array(new SharedArrayBuffer(Int16Array.BYTES_PER_ELEMENT * 2));
    this.inputRotation = new Int16Array(new SharedArrayBuffer(Int16Array.BYTES_PER_ELEMENT * 2));
    this.userPosition = new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 3));
    this.userRotation = new Int16Array(new SharedArrayBuffer(Int16Array.BYTES_PER_ELEMENT * 4));
    this.cameraPosition = new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 3));
    this.cameraYaw = new Int16Array(new SharedArrayBuffer(Int16Array.BYTES_PER_ELEMENT));

    this.behavior = new BehaviorModule(this);
    this.input = new InputModule(this);
    this.physics = new PhysicsModule(this);
    this.player = new PlayerModules(this);
    this.render = new RenderModule(this);
    this.scene = new SceneModule(this);
  }

  get controls() {
    return this.#controls;
  }

  set controls(value: ControlsType) {
    if (value === this.#controls) return;
    this.#controls = value;
    this.input.keyboard.controls = value;
    this.render.send({ subject: "set_controls", data: value });
    this.physics.send({ subject: "set_controls", data: value });
  }

  get visuals() {
    return this.#visuals;
  }

  set visuals(value: boolean) {
    if (value === this.#visuals) return;
    this.#visuals = value;
    this.render.send({ subject: "toggle_visuals", data: value });
    this.physics.send({ subject: "toggle_visuals", data: value });
  }

  destroy() {
    this.render.destroy();
    this.input.destroy();
    this.physics.destroy();
    this.behavior.destroy();
  }
}

import { AudioModule } from "./audio/AudioModule";
import { BehaviorModule } from "./behavior/BehaviorModule";
import { DEFAULT_CONTROLS } from "./constants";
import { InputModule } from "./input/InputModule";
import { PhysicsModule } from "./physics/PhysicsModule";
import { PlayerModules } from "./player/PlayerModule";
import { RenderModule } from "./render/RenderModule";
import { SceneModule } from "./scene/SceneModule";
import { ControlsType } from "./types";

/**
 * The main engine class.
 *
 * Uses two canvases, one for rendering and one for UI elements.
 * Requires {@link https://web.dev/cross-origin-isolation-guide/ cross-origin isolation} to be enabled.
 */
export class Engine {
  #canvas: HTMLCanvasElement | OffscreenCanvas | null = null;
  #overlayCanvas: HTMLCanvasElement | null = null;

  readonly audio: AudioModule;
  readonly behavior: BehaviorModule;
  readonly input: InputModule;
  physics: PhysicsModule;
  readonly player: PlayerModules;
  readonly render: RenderModule;
  readonly scene: SceneModule;

  inputPosition: Int16Array;
  inputRotation: Int16Array;
  userPosition: Int32Array;
  userRotation: Int16Array;
  cameraPosition: Int32Array;
  cameraYaw: Int16Array;

  #isPlaying = false;
  #controls: ControlsType = DEFAULT_CONTROLS;
  #showColliders = false;
  #showBVH = false;

  constructor() {
    this.inputPosition = new Int16Array(new SharedArrayBuffer(Int16Array.BYTES_PER_ELEMENT * 2));
    this.inputRotation = new Int16Array(new SharedArrayBuffer(Int16Array.BYTES_PER_ELEMENT * 2));
    this.userPosition = new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 3));
    this.userRotation = new Int16Array(new SharedArrayBuffer(Int16Array.BYTES_PER_ELEMENT * 4));
    this.cameraPosition = new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 3));
    this.cameraYaw = new Int16Array(new SharedArrayBuffer(Int16Array.BYTES_PER_ELEMENT));

    this.audio = new AudioModule(this);
    this.behavior = new BehaviorModule(this);
    this.input = new InputModule(this);
    this.physics = new PhysicsModule(this);
    this.player = new PlayerModules(this);
    this.render = new RenderModule(this);
    this.scene = new SceneModule(this);
  }

  get canvas() {
    return this.#canvas;
  }

  set canvas(value: HTMLCanvasElement | OffscreenCanvas | null) {
    if (value === this.#canvas) return;

    // Convert to OffscreenCanvas if possible
    if (value instanceof HTMLCanvasElement && typeof OffscreenCanvas === "undefined") {
      const offscreen = value.transferControlToOffscreen();
      this.#canvas = offscreen;
      this.render.send({ subject: "set_canvas", data: offscreen }, [offscreen]);
    } else if (value instanceof OffscreenCanvas) {
      this.#canvas = value;
      this.render.send({ subject: "set_canvas", data: value }, [value]);
    } else {
      this.#canvas = value;
      if (this.render.renderThread) this.render.renderThread.canvas = value;
    }

    if (value) {
      this.render.send({ subject: "set_size", data: { width: value.width, height: value.height } });
    }
  }

  get overlayCanvas() {
    return this.#overlayCanvas;
  }

  set overlayCanvas(value: HTMLCanvasElement | null) {
    if (value === this.#overlayCanvas) return;
    this.#overlayCanvas = value;
    this.input.canvas = value;
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

  get showColliders() {
    return this.#showColliders;
  }

  set showColliders(value: boolean) {
    if (value === this.#showColliders) return;
    this.#showColliders = value;
    this.render.send({ subject: "toggle_collider_visuals", data: value });
    this.physics.send({ subject: "toggle_collider_visuals", data: value });
  }

  get showBVH() {
    return this.#showBVH;
  }

  set showBVH(value: boolean) {
    if (value === this.#showBVH) return;
    this.#showBVH = value;
    this.render.send({ subject: "toggle_bvh_visuals", data: value });
  }

  get isPlaying() {
    return this.#isPlaying;
  }

  start() {
    if (this.#isPlaying) return;
    this.#isPlaying = true;
    this.physics.send({ subject: "start", data: null });
  }

  stop() {
    if (!this.#isPlaying) return;
    this.#isPlaying = false;
    this.physics.send({ subject: "stop", data: null });
  }

  /**
   * Used to refresh the engine (such as during a hot reload).
   * The physics thread needs to be destroyed and recreated, something to do with wasm.
   */
  async reset() {
    const wasPlaying = this.#isPlaying;

    this.stop();

    this.render.send({
      subject: "set_transform_controls_target",
      data: { nodeId: null, attach: false },
    });

    this.scene.clear();

    await this.physics.reset();

    if (wasPlaying) this.start();
  }

  destroy() {
    this.stop();

    this.input.destroy();
    this.behavior.destroy();
    this.audio.destroy();

    this.scene.clear();

    this.render.destroy();
    this.physics.destroy();
  }
}

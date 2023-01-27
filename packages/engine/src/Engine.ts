import { DEFAULT_CONTROLS, DEFAULT_VISUALS } from "./constants";
import { InputModule } from "./input/InputModule";
import { PhysicsModule } from "./physics/PhysicsModule";
import { RenderModule } from "./render/RenderModule";
import { SceneModule } from "./scene/SceneModule";

export type ControlsType = "orbit" | "player";

export interface EngineOptions {
  canvas: HTMLCanvasElement;
}

export class Engine {
  readonly canvas: HTMLCanvasElement;

  readonly input: InputModule;
  readonly physics: PhysicsModule;
  readonly render: RenderModule;
  readonly scene: SceneModule;

  #controls: ControlsType = DEFAULT_CONTROLS;
  #visuals = DEFAULT_VISUALS;

  constructor({ canvas }: EngineOptions) {
    this.canvas = canvas;
    this.input = new InputModule(this);
    this.physics = new PhysicsModule(this);
    this.render = new RenderModule(this);
    this.scene = new SceneModule(this);
  }

  get controls() {
    return this.#controls;
  }

  set controls(value: ControlsType) {
    this.#controls = value;
    this.input.keyboard.controls = value;
    this.render.send({ subject: "set_controls", data: value });
    this.physics.send({ subject: "set_controls", data: value });
  }

  get visuals() {
    return this.#visuals;
  }

  set visuals(value: boolean) {
    this.#visuals = value;
    this.render.send({ subject: "toggle_visuals", data: { enabled: value } });
  }

  destroy() {
    this.render.destroy();
    this.input.destroy();
    this.physics.destroy();
  }
}

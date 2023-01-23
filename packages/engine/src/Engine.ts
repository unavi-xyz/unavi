import { DEFAULT_CONTROLS, DEFAULT_VISUALS } from "./constants";
import { InputModule } from "./input/InputModule";
import { PhysicsModule } from "./physics/PhysicsModule";
import { PlayerModule } from "./player/PlayerModule";
import { RenderModule } from "./render/RenderModule";
import { SceneModule } from "./scene/SceneModule";

interface ModuleContainer {
  input: InputModule;
  physics: PhysicsModule;
  player: PlayerModule;
  render: RenderModule;
  scene: SceneModule;
}

export type ControlsType = "orbit" | "player";

export interface EngineOptions {
  canvas: HTMLCanvasElement;
}

export class Engine {
  modules: ModuleContainer;

  #controls: ControlsType = DEFAULT_CONTROLS;
  #visuals = DEFAULT_VISUALS;

  constructor({ canvas }: EngineOptions) {
    const render = new RenderModule(canvas, this);
    const input = new InputModule(canvas, render);
    const physics = new PhysicsModule(input, render);
    const scene = new SceneModule(render, physics);
    const player = new PlayerModule();

    this.modules = {
      input,
      physics,
      player,
      render,
      scene,
    };
  }

  get controls() {
    return this.#controls;
  }

  set controls(value: ControlsType) {
    this.#controls = value;
    this.modules.input.keyboard.controls = value;
    this.modules.render.toRenderThread({ subject: "set_controls", data: value });
    this.modules.physics.toPhysicsThread({ subject: "set_controls", data: value });
  }

  get visuals() {
    return this.#visuals;
  }

  set visuals(value: boolean) {
    this.#visuals = value;
    this.modules.render.toRenderThread({ subject: "toggle_visuals", data: { enabled: value } });
  }

  destroy() {
    this.modules.render.destroy();
    this.modules.input.destroy();
    this.modules.physics.destroy();
  }
}

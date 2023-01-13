import { DEFAULT_CONTROLS } from "./constants";
import { InputModule } from "./input/InputModule";
import { PhysicsModule } from "./physics/PhysicsModule";
import { RenderModule } from "./render/RenderModule";
import { SceneModule } from "./scene/SceneModule";

interface ModuleContainer {
  input: InputModule;
  physics: PhysicsModule;
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

  constructor({ canvas }: EngineOptions) {
    const render = new RenderModule(canvas, this);
    const input = new InputModule(canvas, render);
    const physics = new PhysicsModule(input, render);
    const scene = new SceneModule(render, physics);

    this.modules = {
      input,
      physics,
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

  destroy() {
    this.modules.input.destroy();
  }
}

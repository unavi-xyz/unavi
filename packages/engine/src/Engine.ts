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

  #controls: ControlsType = "player";

  constructor({ canvas }: EngineOptions) {
    const physics = new PhysicsModule();
    const render = new RenderModule(canvas, this);
    const input = new InputModule(canvas, render);
    const scene = new SceneModule(render);

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
    this.modules.render.toRenderThread({ subject: "set_controls", data: value });
  }

  destroy() {
    this.modules.input.destroy();
  }
}

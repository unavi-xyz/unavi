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

export interface EngineOptions {
  canvas: HTMLCanvasElement;
}

export class Engine {
  modules: ModuleContainer;

  constructor({ canvas }: EngineOptions) {
    const physics = new PhysicsModule();
    const render = new RenderModule(canvas);
    const input = new InputModule(canvas, render);
    const scene = new SceneModule(render);

    this.modules = {
      input,
      physics,
      render,
      scene,
    };

    this.modules.scene.load("/models/Cyberia.glb");
  }

  destroy() {
    this.modules.input.destroy();
  }
}

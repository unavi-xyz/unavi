import { PhysicsModule } from "./physics/PhysicsModule";
import { RenderModule } from "./render/RenderModule";
import { SceneModule } from "./scene/SceneModule";

interface ModuleContainer {
  render: RenderModule;
  scene: SceneModule;
  physics: PhysicsModule;
}

export interface EngineOptions {
  canvas: HTMLCanvasElement;
}

export class Engine {
  modules: ModuleContainer;

  constructor({ canvas }: EngineOptions) {
    const render = new RenderModule(canvas);
    const physics = new PhysicsModule();
    const scene = new SceneModule(render);

    this.modules = {
      render,
      physics,
      scene,
    };

    this.modules.scene.load("/models/Cyberia.glb");
  }
}

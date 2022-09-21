import { Scene } from "./scene/Scene";

export interface EngineOptions {
  canvas: HTMLCanvasElement;
  skyboxPath?: string;
  camera?: "orbit" | "player";
  enableTransformControls?: boolean;
  preserveDrawingBuffer?: boolean;
}

export class Engine {
  scene = new Scene();

  constructor({ canvas, camera = "player" }: EngineOptions) {}

  start() {}

  stop() {}

  destroy() {}
}

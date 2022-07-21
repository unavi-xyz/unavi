import { LoadedGLTF } from "./gltf";

type WorkerMessage<T, D> = {
  type: T;
  data: D;
};

// Render
export type RenderStartMessage = WorkerMessage<"start", {}>;
export type RenderStopMessage = WorkerMessage<"stop", {}>;

export type RenderInitMessage = WorkerMessage<
  "renderInit",
  {
    canvas: HTMLCanvasElement;
  }
>;

export type RenderMessage = RenderStartMessage | RenderStopMessage | RenderInitMessage;

// Game
export type GameLoadGltfMessage = WorkerMessage<
  "loadGltf",
  {
    url: string;
  }
>;

export type GameMessage = GameLoadGltfMessage;

// Main
export type MainGLTFMessage = WorkerMessage<"gltf", LoadedGLTF>;

export type MainMessage = MainGLTFMessage;

import { LoadedGLTF } from "./gltf";

type WorkerMessage<T, D> = {
  id: number;
  type: T;
  data: D;
};

// To game worker
export type ToGameWorkerLoadGltf = WorkerMessage<
  "load_gltf",
  {
    uri: string;
  }
>;

export type ToGameMessage = ToGameWorkerLoadGltf;

// From game worker
export type FromGameLoadedGltf = WorkerMessage<"loaded_gltf", LoadedGLTF>;

export type FromGameMessage = FromGameLoadedGltf;

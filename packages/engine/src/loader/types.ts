import { TypedArray } from "bitecs";

import { WorkerMessage } from "../types";

export type Assets = {
  names: string[];
  images: ImageBitmap[];
  accessors: TypedArray[];
};

export type LoadedGLTF = {
  world: ArrayBuffer;
  assets: Assets;
};

export type ToLoaderMessage = WorkerMessage<"load_gltf", string>;

export type FromLoaderLoadedGLTF = WorkerMessage<"gltf_loaded", LoadedGLTF>;
export type FromLoaderMessage = WorkerMessage<"ready"> | FromLoaderLoadedGLTF;

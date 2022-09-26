import { SceneJSON, SceneMessage } from "../scene";
import { WorkerMessage } from "../types";

export type ToLoaderMessage =
  | SceneMessage
  | WorkerMessage<
      "load_gltf",
      {
        id: string;
        uri: string;
      }
    >;

export type FromLoaderMessage =
  | SceneMessage
  | WorkerMessage<"ready">
  | WorkerMessage<
      "gltf_loaded",
      {
        id: string;
        scene: SceneJSON;
      }
    >;

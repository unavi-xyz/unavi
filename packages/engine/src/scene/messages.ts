import { WorkerMessage } from "../types";
import { AccessorJSON } from "./utils/AccessorUtils";
import { BufferJSON } from "./utils/BufferUtils";
import { MaterialJSON } from "./utils/MaterialUtils";
import { MeshJSON } from "./utils/MeshUtils";
import { NodeJSON } from "./utils/NodeUtils";
import { PrimitiveJSON } from "./utils/PrimitiveUtils";
import { TextureJSON } from "./utils/TextureUtils";

const subjects = [
  "create_buffer",
  "dispose_buffer",

  "create_accessor",
  "dispose_accessor",

  "create_texture",
  "dispose_texture",

  "create_material",
  "dispose_material",

  "create_primitive",
  "dispose_primitive",

  "create_mesh",
  "dispose_mesh",

  "create_node",
  "dispose_node",
] as const;

type SceneMessageSubject = (typeof subjects)[number];
type SceneWorkerMessage<S extends SceneMessageSubject, D> = WorkerMessage<S, D>;

export type SceneMessage =
  // Buffer
  | SceneWorkerMessage<"create_buffer", { id: string; json: BufferJSON }>
  | SceneWorkerMessage<"dispose_buffer", string>
  // Accessor
  | SceneWorkerMessage<"create_accessor", { id: string; json: AccessorJSON }>
  | SceneWorkerMessage<"dispose_accessor", string>
  // Texture
  | SceneWorkerMessage<"create_texture", { id: string; json: TextureJSON }>
  | SceneWorkerMessage<"dispose_texture", string>
  // Material
  | SceneWorkerMessage<"create_material", { id: string; json: MaterialJSON }>
  | SceneWorkerMessage<"dispose_material", string>
  // Primitive
  | SceneWorkerMessage<"create_primitive", { id: string; json: PrimitiveJSON }>
  | SceneWorkerMessage<"dispose_primitive", string>
  // Mesh
  | SceneWorkerMessage<"create_mesh", { id: string; json: MeshJSON }>
  | SceneWorkerMessage<"dispose_mesh", string>
  // Node
  | SceneWorkerMessage<"create_node", { id: string; json: NodeJSON }>
  | SceneWorkerMessage<"dispose_node", string>;

export function isSceneMessage(message: WorkerMessage): message is SceneMessage {
  return subjects.includes(message.subject as SceneMessageSubject);
}

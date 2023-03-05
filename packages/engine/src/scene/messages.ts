import { MessageJSON } from "../types";
import { AccessorJSON } from "./attributes/Accessors";
import { AnimationJSON } from "./attributes/Animations";
import { BufferJSON } from "./attributes/Buffers";
import { MaterialJSON } from "./attributes/Materials";
import { MeshJSON } from "./attributes/Meshes";
import { NodeJSON } from "./attributes/Nodes";
import { PrimitiveJSON } from "./attributes/Primitives";
import { SkinJSON } from "./attributes/Skins";
import { TextureJSON } from "./attributes/Textures";

const subjects = [
  "create_buffer",
  "dispose_buffer",

  "create_accessor",
  "dispose_accessor",

  "create_texture",
  "dispose_texture",

  "create_material",
  "change_material",
  "dispose_material",

  "create_primitive",
  "change_primitive",
  "dispose_primitive",

  "create_mesh",
  "change_mesh",
  "dispose_mesh",

  "create_node",
  "change_node",
  "dispose_node",

  "create_skin",
  "dispose_skin",

  "create_animation",
  "change_animation",
  "dispose_animation",
] as const;

type SceneMessageSubject = (typeof subjects)[number];
type SceneWorkerMessage<S extends SceneMessageSubject, D> = MessageJSON<S, D>;

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
  | SceneWorkerMessage<"change_material", { id: string; json: Partial<MaterialJSON> }>
  | SceneWorkerMessage<"dispose_material", string>
  // Primitive
  | SceneWorkerMessage<"create_primitive", { id: string; json: PrimitiveJSON }>
  | SceneWorkerMessage<"change_primitive", { id: string; json: Partial<PrimitiveJSON> }>
  | SceneWorkerMessage<"dispose_primitive", string>
  // Mesh
  | SceneWorkerMessage<"create_mesh", { id: string; json: MeshJSON }>
  | SceneWorkerMessage<"change_mesh", { id: string; json: Partial<MeshJSON> }>
  | SceneWorkerMessage<"dispose_mesh", string>
  // Node
  | SceneWorkerMessage<"create_node", { id: string; json: Partial<NodeJSON> }>
  | SceneWorkerMessage<"change_node", { id: string; json: Partial<NodeJSON> }>
  | SceneWorkerMessage<"dispose_node", string>
  // Skin
  | SceneWorkerMessage<"create_skin", { id: string; json: SkinJSON }>
  | SceneWorkerMessage<"dispose_skin", string>
  // Animation
  | SceneWorkerMessage<"create_animation", { id: string; json: AnimationJSON }>
  | SceneWorkerMessage<"change_animation", { id: string; json: Partial<AnimationJSON> }>
  | SceneWorkerMessage<"dispose_animation", string>;

export function isSceneMessage(message: MessageJSON): message is SceneMessage {
  return subjects.includes(message.subject as SceneMessageSubject);
}

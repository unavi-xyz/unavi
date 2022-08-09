import { AnimationAction, Group } from "three";

import { LoadedGLTF } from "./gltf";

export interface IGLTF {
  scene: Group;
  animations: AnimationAction[];
}

type WorkerMessage<S extends string, D> = {
  id?: number;
  subject: S;
  data: D;
};

// To render worker
export type ToRenderDestroy = WorkerMessage<"destroy", null>;
export type ToRenderAddObject = WorkerMessage<
  "add_object",
  {
    json: any;
  }
>;
export type ToRenderGetObject = WorkerMessage<
  "get_object",
  {
    uuid: string;
  }
>;
export type ToRenderCreateSkybox = WorkerMessage<
  "create_skybox",
  {
    path: string;
  }
>;

// Orbit controls
export type ToRenderCreateOrbitControls = WorkerMessage<"create_orbit_controls", null>;
export type ToRenderDestroyOrbitControls = WorkerMessage<"destroy_orbit_controls", null>;
export type ToRenderSetOrbitControlsEnabled = WorkerMessage<
  "set_orbit_controls_enabled",
  { enabled: boolean }
>;

// Transform controls
export type ToRenderCreateTransformControls = WorkerMessage<"create_transform_controls", null>;
export type ToRenderDestroyTransformControls = WorkerMessage<"destroy_transform_controls", null>;
export type ToRenderSetTransformControlsEnabled = WorkerMessage<
  "set_transform_controls_enabled",
  { enabled: boolean }
>;
export type ToRenderSetTransformControlsMode = WorkerMessage<
  "set_transform_controls_mode",
  { mode: "translate" | "rotate" | "scale" }
>;
export type ToRenderAttachTransformControls = WorkerMessage<
  "attach_transform_controls",
  { uuid: string }
>;
export type ToRenderDetachTransformControls = WorkerMessage<"detach_transform_controls", null>;

export type ToRenderMessage =
  | ToRenderDestroy
  | ToRenderAddObject
  | ToRenderGetObject
  | ToRenderCreateSkybox
  | ToRenderCreateOrbitControls
  | ToRenderDestroyOrbitControls
  | ToRenderSetOrbitControlsEnabled
  | ToRenderCreateTransformControls
  | ToRenderDestroyTransformControls
  | ToRenderSetTransformControlsEnabled
  | ToRenderSetTransformControlsMode
  | ToRenderAttachTransformControls
  | ToRenderDetachTransformControls;

// From render worker
export type FromRenderClickIntersections = WorkerMessage<
  "click_intersection",
  {
    uuid: string | null;
  }
>;
export type FromRenderGotObject = WorkerMessage<
  "got_object",
  {
    json: any;
  }
>;

export type FromRenderMessage = FromRenderClickIntersections | FromRenderGotObject;

// To game worker
export type ToGameLoadGltf = WorkerMessage<
  "load_gltf",
  {
    uri: string;
  }
>;
export type ToGameMessage = ToGameLoadGltf;

// From game worker
export type FromGameLoadedGltf = WorkerMessage<"loaded_gltf", LoadedGLTF>;
export type FromGameMessage = FromGameLoadedGltf;

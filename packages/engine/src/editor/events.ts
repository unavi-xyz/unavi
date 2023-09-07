import {
  EditNode_Collider_Type,
  EditNode_RigidBody_Type,
} from "@unavi/protocol";
import { Resource } from "lattice-engine/core";
import { Transform } from "lattice-engine/scene";
import { f32, struct } from "thyseus";

@struct
export class AddNode {
  name: string = "";
}

@struct
export class EditNode {
  target: string = "";

  name: string = "";
  meshName: string = "";
  parentName: string = "";

  translation: boolean = false;
  rotation: boolean = false;
  scale: boolean = false;
  transform: Transform = new Transform();
}

@struct
export class EditExtra {
  target: string = "";
  key: string = "";
  value: string = "";
}

@struct
export class EditRigidBody {
  target: string = "";
  type: EditNode_RigidBody_Type = EditNode_RigidBody_Type.NONE;
}

@struct
export class EditCollider {
  target: string = "";
  type: EditNode_Collider_Type = EditNode_Collider_Type.NONE;
  size: [f32, f32, f32] = [0, 0, 0];
  height: f32 = 0;
  radius: f32 = 0;
  mesh: string = "";
}

@struct
export class AddMesh {
  name: string = "";
}

@struct
export class EditMesh {
  target: string = "";

  name: string = "";
  material: string = "";

  indices: Resource<Uint32Array> = new Resource();
  colors: Resource<Float32Array> = new Resource();
  joints: Resource<Float32Array> = new Resource();
  normals: Resource<Float32Array> = new Resource();
  positions: Resource<Float32Array> = new Resource();
  uv1: Resource<Float32Array> = new Resource();
  uv2: Resource<Float32Array> = new Resource();
  uv3: Resource<Float32Array> = new Resource();
  uv: Resource<Float32Array> = new Resource();
  weights: Resource<Float32Array> = new Resource();
}

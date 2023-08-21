import { Resource } from "lattice-engine/core";
import { Transform } from "lattice-engine/scene";
import { f32, struct, u8 } from "thyseus";

@struct
export class AddNode {
  name: Resource<string> = new Resource();
}

@struct
export class EditNode {
  target: Resource<string> = new Resource();
  name: Resource<string> = new Resource();
  meshName: Resource<string> = new Resource();
  parentName: Resource<string> = new Resource();

  translation: boolean = false;
  rotation: boolean = false;
  scale: boolean = false;
  transform: Transform = new Transform();
}

@struct
export class EditExtra {
  target: Resource<string> = new Resource();
  key: Resource<string> = new Resource();
  value: Resource<string> = new Resource();
}

@struct
export class EditRigidBody {
  target: Resource<string> = new Resource();
  type: u8 = 0;
}

@struct
export class EditCollider {
  target: Resource<string> = new Resource();
  type: u8 = 0;
  size: Resource<Float32Array> = new Resource();
  height: f32 = 0;
  radius: f32 = 0;
  mesh: Resource<string> = new Resource();
}

@struct
export class AddMesh {
  name: Resource<string> = new Resource();
}

@struct
export class EditMesh {
  target: Resource<string> = new Resource();

  name: Resource<string> = new Resource();
  material: Resource<string> = new Resource();

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

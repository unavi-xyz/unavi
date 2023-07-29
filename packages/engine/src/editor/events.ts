import { ColliderType, RigidBodyType } from "@unavi/protocol";
import { Resource } from "lattice-engine/core";
import { Transform } from "lattice-engine/scene";
import { struct } from "thyseus";

@struct
export class AddNode {
  @struct.string declare name: string;
}

@struct
export class EditNode {
  @struct.string declare target: string;

  @struct.string declare name: string;
  @struct.string declare meshName: string;
  @struct.string declare parentName: string;

  @struct.bool declare translation: boolean;
  @struct.bool declare rotation: boolean;
  @struct.bool declare scale: boolean;
  @struct.substruct(Transform) declare transform: Transform;
}

@struct
export class EditExtra {
  @struct.string declare target: string;

  @struct.string declare key: string;
  @struct.string declare value: string;
}

@struct
export class EditRigidBody {
  @struct.string declare target: string;
  @struct.string declare type: RigidBodyType | "none";
}

@struct
export class EditCollider {
  @struct.string declare target: string;
  @struct.string declare type: ColliderType | "none";
  @struct.array({ length: 3, type: "f32" }) declare size: Float32Array;
  @struct.f32 declare height: number;
  @struct.f32 declare radius: number;
  @struct.f32 declare meshName: string;
}

@struct
export class AddMesh {
  @struct.string declare name: string;
}

@struct
export class EditMesh {
  @struct.string declare target: string;

  @struct.string declare name: string;
  @struct.string declare materialName: string;

  @struct.substruct(Resource) declare indices: Resource<Uint32Array>;
  @struct.substruct(Resource) declare colors: Resource<Float32Array>;
  @struct.substruct(Resource) declare joints: Resource<Float32Array>;
  @struct.substruct(Resource) declare normals: Resource<Float32Array>;
  @struct.substruct(Resource) declare positions: Resource<Float32Array>;
  @struct.substruct(Resource) declare uv1: Resource<Float32Array>;
  @struct.substruct(Resource) declare uv2: Resource<Float32Array>;
  @struct.substruct(Resource) declare uv3: Resource<Float32Array>;
  @struct.substruct(Resource) declare uv: Resource<Float32Array>;
  @struct.substruct(Resource) declare weights: Resource<Float32Array>;
}

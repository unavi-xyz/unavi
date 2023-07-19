import { Resource } from "lattice-engine/core";
import { struct } from "thyseus";

@struct
export class AddNode {
  @struct.string declare name: string;
  @struct.string declare meshName: string;
  @struct.string declare parentName: string;
}

@struct
export class AddMesh {
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

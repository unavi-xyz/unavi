import { Types, defineComponent, defineDeserializer, defineSerializer } from "bitecs";

export enum ColliderTypes {
  "box",
  "sphere",
  "capsule",
  "hull",
  "mesh",
  "compound",
}

export enum RigidBodyTypes {
  "static",
  "dynamic",
  "kinematic",
  "trigger",
}

const Vector3 = {
  x: Types.f32,
  y: Types.f32,
  z: Types.f32,
};

const Vector4 = {
  x: Types.f32,
  y: Types.f32,
  z: Types.f32,
  w: Types.f32,
};

// Assets are references to data not stored in the ECS
const Asset = Types.ui32;

// Node
export const Node = defineComponent({
  name: Asset,
  position: Vector3,
  rotation: Vector4,
  scale: Vector3,
  weights: [Types.f32, 3],
});
export const NodeParent = defineComponent({ parent: Types.eid });
export const NodeMesh = defineComponent({ mesh: Types.eid });

// Primitive
export const AttributeType = {
  array: Asset,
  itemSize: Types.ui32,
  normalized: Types.ui8,
};

export const Primitive = defineComponent({ mesh: Types.eid, mode: Types.ui8 });
export const PrimitiveMaterial = defineComponent({ material: Types.eid });
export const PrimitiveIndices = defineComponent(AttributeType);
export const AttributePosition = defineComponent(AttributeType);
export const AttributeNormal = defineComponent(AttributeType);
export const AttributeTangent = defineComponent(AttributeType);
export const AttributeColor = defineComponent(AttributeType);
export const AttributeUV = defineComponent(AttributeType);
export const AttributeUV2 = defineComponent(AttributeType);
export const AttributeSkinWeight = defineComponent(AttributeType);
export const AttributeSkinIndex = defineComponent(AttributeType);
export const MorphTarget = defineComponent({ primitive: Types.eid });

// Material
export enum AlphaMode {
  "OPAQUE",
  "BLEND",
  "MASK",
}

export const TextureInfo = {
  magFilter: Types.ui16,
  minFilter: Types.ui16,
  texCoord: Types.ui16,
  wrapS: Types.ui16,
  wrapT: Types.ui16,
};

export const Material = defineComponent({
  name: Asset,
  doubleSided: Types.ui8,

  alpha: Types.f32,
  alphaCutoff: Types.f32,
  alphaMode: Types.ui8,

  baseColorFactor: Vector4,
  baseColorHex: Types.ui32,

  emissiveFactor: Vector3,
  emissiveHex: Types.ui32,

  metallicFactor: Types.f32,
  roughnessFactor: Types.f32,

  normalScale: Types.f32,

  occlusionStrength: Types.f32,
});

export const ColorTexture = defineComponent({ texture: Types.eid, info: TextureInfo });
export const EmissiveTexture = defineComponent({ texture: Types.eid, info: TextureInfo });
export const MetallicRoughnessTexture = defineComponent({ texture: Types.eid, info: TextureInfo });
export const NormalTexture = defineComponent({ texture: Types.eid, info: TextureInfo });
export const OcclusionTexture = defineComponent({ texture: Types.eid, info: TextureInfo });

export const Texture = defineComponent({
  name: Asset,
  image: Asset,
});

// Animation
export const Animation = defineComponent({
  name: Asset,
});

export enum TargetPath {
  "translation",
  "rotation",
  "scale",
  "weights",
}

export const AnimationChannel = defineComponent({
  animation: Types.eid,
  sampler: Types.eid,
  target: Types.eid,
  targetPath: Types.ui8,
});

export enum Interpolation {
  "STEP",
  "LINEAR",
  "CUBICSPLINE",
}

export const AnimationSampler = defineComponent({
  input: Asset,
  output: Asset,
  interpolation: Types.ui8,
});

export const config = [
  Node,
  NodeParent,
  NodeMesh,
  Primitive,
  PrimitiveMaterial,
  PrimitiveIndices,
  AttributePosition,
  AttributeNormal,
  AttributeTangent,
  AttributeColor,
  AttributeUV,
  AttributeUV2,
  AttributeSkinWeight,
  AttributeSkinIndex,
  MorphTarget,
  Material,
  ColorTexture,
  EmissiveTexture,
  MetallicRoughnessTexture,
  NormalTexture,
  OcclusionTexture,
  Texture,
  Animation,
  AnimationChannel,
  AnimationSampler,
];
export const serialize = defineSerializer(config);
export const deserialize = defineDeserializer(config);

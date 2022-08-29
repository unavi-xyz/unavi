import { Types, defineComponent } from "bitecs";

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

// AssetRefs are references to data stored outside of the ECS
const AssetRef = Types.ui32;

// Node
export const Node = defineComponent({
  name: AssetRef,
  position: Vector3,
  rotation: Vector4,
  scale: Vector3,
  weights: [Types.f32, 3],
});
export const NodeParent = defineComponent({ parent: Types.eid });
export const NodeMesh = defineComponent({ mesh: Types.eid });
export const NodeSkin = defineComponent({ skin: Types.eid });

// Skin
export const Skin = defineComponent({
  inverseBindMatrices: AssetRef,
});

export const SkinJoint = defineComponent({
  skin: Types.eid,
  bone: Types.eid,
});

// Primitive
export const AttributeType = {
  array: AssetRef,
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
  name: AssetRef,
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
  name: AssetRef,
  image: AssetRef,
});

// Animation
export const Animation = defineComponent({
  name: AssetRef,
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
  input: AssetRef,
  output: AssetRef,
  interpolation: Types.ui8,
});

export const gltfConfig = [
  Node,
  NodeParent,
  NodeMesh,
  NodeSkin,
  Skin,
  SkinJoint,
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

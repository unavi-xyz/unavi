import { Types, defineComponent, defineDeserializer, defineSerializer } from "bitecs";

import { gltfConfig } from "./glTF-components";

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

// Scene Objects
export const SceneObject = defineComponent({
  name: Types.ui32,
  position: Vector3,
  rotation: Vector4,
  scale: Vector3,
});
export const Child = defineComponent({ parent: Types.eid });
export const MeshMaterial = defineComponent({ material: Types.eid });
export const Box = defineComponent({ width: Types.f32, height: Types.f32, depth: Types.f32 });
export const Sphere = defineComponent({
  radius: Types.f32,
  widthSegments: Types.ui32,
  heightSegments: Types.ui32,
});
export const Cylinder = defineComponent({
  radiusTop: Types.f32,
  radiusBottom: Types.f32,
  height: Types.f32,
  radialSegments: Types.ui32,
});
export const GLTF = defineComponent({ gltf: Types.ui32 });

export const sceneConfig = [SceneObject, Child, MeshMaterial, Box, Sphere, Cylinder, GLTF];
export const config = [...gltfConfig, ...sceneConfig];

export const serialize = defineSerializer(config);
export const deserialize = defineDeserializer(config);

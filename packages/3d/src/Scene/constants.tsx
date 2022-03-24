import { CoreProperties, Properties } from "./types";

import { Box, boxDefaultProperties } from "./objects/Box";
import { Sphere, sphereDefaultProperties } from "./objects/Sphere";
import { gltfDefaultProperties, GLTFModel } from "./objects/GLTFModel";
import { Heightmap } from "./objects/Heightmap";

type GenericSceneObject = {
  [key: string]: {
    limit?: number;
    properties: CoreProperties & Partial<Properties>;
    component: JSX.Element;
  };
};

function generify<T extends GenericSceneObject>(obj: T) {
  return obj;
}

export const SceneObjects = generify({
  Box: {
    properties: boxDefaultProperties,
    component: <Box />,
  },
  Sphere: {
    properties: sphereDefaultProperties,
    component: <Sphere />,
  },
  GLTF: {
    properties: gltfDefaultProperties,
    component: <GLTFModel />,
  },
  Heightmap: {
    properties: gltfDefaultProperties,
    component: <Heightmap />,
  },
});

import Box, { boxDefaultParams } from "./primitives/Box";
import Sphere, { sphereDefaultParams } from "./primitives/Sphere";

export const PRIMITIVES = {
  Box: { Component: Box, default: boxDefaultParams },
  Sphere: { Component: Sphere, default: sphereDefaultParams },
};

export type Primitive = keyof typeof PRIMITIVES;

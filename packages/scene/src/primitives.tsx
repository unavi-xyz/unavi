import Box, { boxDefaultParams } from "./primitives/Box";
import Sphere, { sphereDefaultParams } from "./primitives/Sphere";

export type Primitive = "Box" | "Sphere" | "Group";

export const PRIMITIVES = {
  Box: { Component: Box, default: boxDefaultParams },
  Sphere: { Component: Sphere, default: sphereDefaultParams },
  Group: { Component: undefined, default: undefined },
};

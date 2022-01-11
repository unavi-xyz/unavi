import { SceneObject } from "./SceneObject";
import { Box, Sphere } from "./components";

export enum OBJ_NAMES {
  box = "Box",
  sphere = "Sphere",
}

export const OBJECTS: { [key in OBJ_NAMES]: SceneObject } = {
  [OBJ_NAMES.box]: new SceneObject(OBJ_NAMES.box, <Box />),
  [OBJ_NAMES.sphere]: new SceneObject(OBJ_NAMES.sphere, <Sphere />),
};

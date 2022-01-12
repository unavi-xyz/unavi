import { SceneObject } from "./classes/SceneObject";
import { Box, Sphere } from "./components";

export enum OBJ_NAMES {
  box = "Box",
  sphere = "Sphere",
}

export const OBJECTS: { [key in OBJ_NAMES]: SceneObject } = {
  [OBJ_NAMES.box]: new SceneObject(OBJ_NAMES.box),
  [OBJ_NAMES.sphere]: new SceneObject(OBJ_NAMES.sphere),
};

export function getObjectComponent(object: SceneObject) {
  switch (object.name) {
    case OBJ_NAMES.box:
      return <Box />;
    case OBJ_NAMES.sphere:
      return <Sphere />;
  }
}

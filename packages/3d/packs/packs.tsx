import "@react-three/fiber"; // idk why i need this to stop type errors

import { SceneObject, Box, Sphere } from "..";

export enum OBJECTS {
  Box = "Box",
  Sphere = "Sphere",
  Portal = "Portal",
  Spawn = "Spawn",
}

export const PACKS = {
  Basic: [OBJECTS.Box, OBJECTS.Sphere],
  Game: [OBJECTS.Portal, OBJECTS.Spawn],
};

export function getObjectComponent(object: SceneObject) {
  switch (object.name) {
    case OBJECTS.Box:
      return <Box object={object} />;
    case OBJECTS.Sphere:
      return <Sphere object={object} />;
    case OBJECTS.Portal:
    case OBJECTS.Spawn:
      break;
  }
}

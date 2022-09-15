import { BaseEntity, Box, Cylinder, Sphere } from "@wired-labs/engine";
import { nanoid } from "nanoid";

import { addEntity } from "../../actions/AddEntityAction";

enum ObjectName {
  Box = "Box",
  Sphere = "Sphere",
  Cylinder = "Cylinder",
}

export default function ObjectsMenu() {
  function addObject(name: ObjectName) {
    const entity = createEntity(name);
    addEntity(entity);
  }

  return (
    <div className="space-y-0.5 p-2">
      {Object.values(ObjectName).map((name) => (
        <button
          key={name}
          onClick={() => addObject(name)}
          className="hover:bg-primaryContainer hover:text-onPrimaryContainer flex w-full
                     items-center rounded px-4 py-0.5 transition"
        >
          {name}
        </button>
      ))}
    </div>
  );
}

function createEntity(name: ObjectName) {
  const id = nanoid();
  const base: BaseEntity = {
    id,
    name,
    parent: "root",
    children: [],
    position: [0, 0, 0],
    rotation: [0, 0, 0],
    scale: [1, 1, 1],
  };

  // Add object component
  switch (name) {
    case ObjectName.Box:
      const box: Box = {
        ...base,
        type: "Box",
        width: 1,
        height: 1,
        depth: 1,
      };
      return box;
    case ObjectName.Sphere:
      const sphere: Sphere = {
        ...base,
        type: "Sphere",
        radius: 0.5,
        widthSegments: 32,
        heightSegments: 32,
      };
      return sphere;
    case ObjectName.Cylinder:
      const cylinder: Cylinder = {
        ...base,
        type: "Cylinder",
        radiusTop: 0.5,
        radiusBottom: 0.5,
        height: 1,
        radialSegments: 32,
      };
      return cylinder;
    default:
      throw new Error("Unknown object name");
  }
}

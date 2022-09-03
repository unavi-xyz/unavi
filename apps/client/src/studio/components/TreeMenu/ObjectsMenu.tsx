import { nanoid } from "nanoid";

import { BaseObject, Box, Cylinder, Sphere } from "@wired-labs/engine";

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
    <div className="p-2 space-y-0.5">
      {Object.values(ObjectName).map((name) => (
        <button
          key={name}
          onClick={() => addObject(name)}
          className="w-full flex hover:bg-primaryContainer hover:text-onPrimaryContainer
                     rounded px-4 py-0.5 transition items-center"
        >
          {name}
        </button>
      ))}
    </div>
  );
}

function createEntity(name: ObjectName) {
  const id = nanoid();
  const base: BaseObject = {
    id,
    name,
    parent: null,
    position: [0, 0, 0],
    rotation: [0, 0, 0, 1],
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

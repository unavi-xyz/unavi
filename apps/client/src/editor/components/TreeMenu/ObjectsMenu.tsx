import { BoxMesh, CylinderMesh, Entity, SphereMesh } from "@wired-labs/engine";

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
  const entity = new Entity();
  entity.name = name;

  // Add object component
  switch (name) {
    case ObjectName.Box:
      entity.mesh = new BoxMesh();
      break;
    case ObjectName.Sphere:
      entity.mesh = new SphereMesh();
      break;
    case ObjectName.Cylinder:
      entity.mesh = new CylinderMesh();
      break;
    default:
      throw new Error("Unknown object name");
  }

  return entity;
}

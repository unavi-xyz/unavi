import {
  BoxCollider,
  BoxMesh,
  CylinderCollider,
  CylinderMesh,
  Entity,
  SphereCollider,
  SphereMesh,
} from "@wired-labs/engine";

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
          className="flex w-full items-center rounded
                     px-4 py-0.5 transition hover:bg-primaryContainer hover:text-onPrimaryContainer"
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
      entity.collider = new BoxCollider();
      break;
    case ObjectName.Sphere:
      entity.mesh = new SphereMesh();
      entity.collider = new SphereCollider();
      break;
    case ObjectName.Cylinder:
      entity.mesh = new CylinderMesh();
      entity.collider = new CylinderCollider();
      break;
    default:
      throw new Error("Unknown object name");
  }

  return entity;
}

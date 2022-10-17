import {
  BoxCollider,
  BoxMesh,
  CylinderCollider,
  CylinderMesh,
  Entity,
  GLTFMesh,
  SphereCollider,
  SphereMesh,
} from "@wired-labs/engine";

import { addEntity } from "../../actions/AddEntityAction";
import { useEditorStore } from "../../store";

enum ObjectName {
  Box = "Box",
  Sphere = "Sphere",
  Cylinder = "Cylinder",
  glTF = "glTF Model",
  Group = "Group",
}

export default function ObjectsMenu() {
  function addObject(name: ObjectName) {
    const entity = createEntity(name);
    addEntity(entity);

    // Select new entity
    useEditorStore.setState({ selectedId: entity.id });
  }

  return (
    <div className="space-y-0.5 p-2">
      {Object.values(ObjectName).map((name) => (
        <button
          key={name}
          onClick={() => addObject(name)}
          className="flex w-full items-center rounded-lg px-4 py-0.5 transition hover:bg-primaryContainer hover:text-onPrimaryContainer"
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
    case ObjectName.Box: {
      entity.mesh = new BoxMesh();
      entity.collider = new BoxCollider();
      break;
    }

    case ObjectName.Sphere: {
      entity.mesh = new SphereMesh();
      entity.collider = new SphereCollider();
      break;
    }

    case ObjectName.Cylinder: {
      entity.mesh = new CylinderMesh();
      entity.collider = new CylinderCollider();
      break;
    }

    case ObjectName.glTF: {
      entity.mesh = new GLTFMesh();
      break;
    }

    case ObjectName.Group: {
      break;
    }

    default:
      throw new Error("Unknown object name");
  }

  return entity;
}

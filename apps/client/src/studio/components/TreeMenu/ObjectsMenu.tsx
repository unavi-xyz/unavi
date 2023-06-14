import { Engine, MeshExtras } from "@unavi/engine";

import { DropdownItem } from "../../../ui/DropdownMenu";
import { useStudio } from "../Studio";

const OBJECT_NAME = {
  Box: "Box",
  Cylinder: "Cylinder",
  Empty: "Empty",
  Sphere: "Sphere",
} as const;

type ObjectName = (typeof OBJECT_NAME)[keyof typeof OBJECT_NAME];

export default function ObjectsMenu() {
  const { engine, setSelectedId } = useStudio();

  function addObject(name: ObjectName) {
    if (!engine) return;

    // Create node
    const id = createNode(name, engine);
    // Select new node
    setSelectedId(id);
  }

  return (
    <div className="py-2">
      {Object.values(OBJECT_NAME).map((name) => (
        <DropdownItem
          key={name}
          onClick={() => addObject(name)}
          className="flex w-full cursor-default items-center whitespace-nowrap px-6 outline-none focus:bg-neutral-200 active:opacity-80"
        >
          {name}
        </DropdownItem>
      ))}
    </div>
  );
}

function createNode(name: ObjectName, engine: Engine) {
  const { id, object: node } = engine.scene.node.create();

  node.setName(name);

  switch (name) {
    case OBJECT_NAME.Box: {
      const { object: mesh } = engine.scene.mesh.create();
      node.setMesh(mesh);

      const extras: MeshExtras = {
        customMesh: {
          depth: 1,
          height: 1,
          type: "Box",
          width: 1,
        },
      };

      mesh.setExtras(extras);
      break;
    }

    case OBJECT_NAME.Sphere: {
      const { object: mesh } = engine.scene.mesh.create();
      node.setMesh(mesh);

      const extras: MeshExtras = {
        customMesh: {
          heightSegments: 32,
          radius: 0.5,
          type: "Sphere",
          widthSegments: 32,
        },
      };

      mesh.setExtras(extras);
      break;
    }

    case OBJECT_NAME.Cylinder: {
      const { object: mesh } = engine.scene.mesh.create();
      node.setMesh(mesh);

      const extras: MeshExtras = {
        customMesh: {
          height: 1,
          radialSegments: 32,
          radiusBottom: 0.5,
          radiusTop: 0.5,
          type: "Cylinder",
        },
      };

      mesh.setExtras(extras);
      break;
    }
  }

  return id;
}

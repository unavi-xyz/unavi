import { MeshExtras } from "engine";

import { useEditorStore } from "../../store";

const OBJECT_NAME = {
  Box: "Box",
  Sphere: "Sphere",
  Cylinder: "Cylinder",
  Empty: "Empty",
} as const;

type ObjectName = (typeof OBJECT_NAME)[keyof typeof OBJECT_NAME];

export default function ObjectsMenu() {
  function addObject(name: ObjectName) {
    // Create node
    const id = createNode(name);

    // Select new node
    useEditorStore.setState({ selectedId: id });
  }

  return (
    <div className="py-2">
      {Object.values(OBJECT_NAME).map((name) => (
        <button
          key={name}
          onClick={() => addObject(name)}
          className="flex w-full cursor-default items-center whitespace-nowrap px-6 py-0.5 hover:bg-neutral-200 active:opacity-80"
        >
          {name}
        </button>
      ))}
    </div>
  );
}

function createNode(name: ObjectName) {
  const { engine } = useEditorStore.getState();
  if (!engine) return;

  const { id, object: node } = engine.scene.node.create();

  node.setName(name);

  switch (name) {
    case OBJECT_NAME.Box: {
      const { object: mesh } = engine.scene.mesh.create();
      node.setMesh(mesh);

      const extras: MeshExtras = {
        customMesh: {
          type: "Box",
          width: 1,
          height: 1,
          depth: 1,
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
          type: "Sphere",
          radius: 0.5,
          widthSegments: 32,
          heightSegments: 32,
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
          type: "Cylinder",
          radiusTop: 0.5,
          radiusBottom: 0.5,
          height: 1,
          radialSegments: 32,
        },
      };

      mesh.setExtras(extras);
      break;
    }
  }

  return id;
}

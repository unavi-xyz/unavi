import { MeshExtras } from "engine";

import { useEditorStore } from "../../store";

const OBJECT_NAME = {
  Box: "Box",
  Sphere: "Sphere",
  Cylinder: "Cylinder",
  Model: "GLTF Model",
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
    <div className="space-y-0.5 p-2">
      {Object.values(OBJECT_NAME).map((name) => (
        <button
          key={name}
          onClick={() => addObject(name)}
          className="flex w-full items-center whitespace-nowrap rounded-lg px-4 py-0.5 transition hover:bg-neutral-200"
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

  const { id, object: node } = engine.modules.scene.node.create();
  const { object: mesh } = engine.modules.scene.mesh.create();

  node.setName(name);
  node.setMesh(mesh);

  switch (name) {
    case OBJECT_NAME.Box: {
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

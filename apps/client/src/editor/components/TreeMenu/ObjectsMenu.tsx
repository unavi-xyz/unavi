import {
  BoxCollider,
  BoxMesh,
  CylinderCollider,
  CylinderMesh,
  GLTFMesh,
  MeshCollider,
  Node,
  SphereCollider,
  SphereMesh,
} from "@wired-labs/engine";

import { addNode } from "../../actions/AddNodeAction";
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
    const node = createNode(name);
    addNode(node);

    // Select new node
    useEditorStore.setState({ selectedId: node.id });
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

function createNode(name: ObjectName) {
  const node = new Node();
  node.name = name;

  // Add object component
  switch (name) {
    case ObjectName.Box: {
      node.mesh = new BoxMesh();
      node.collider = new BoxCollider();
      break;
    }

    case ObjectName.Sphere: {
      node.mesh = new SphereMesh();
      node.collider = new SphereCollider();
      break;
    }

    case ObjectName.Cylinder: {
      node.mesh = new CylinderMesh();
      node.collider = new CylinderCollider();
      break;
    }

    case ObjectName.glTF: {
      node.mesh = new GLTFMesh();
      node.collider = new MeshCollider();
      break;
    }

    case ObjectName.Group: {
      break;
    }

    default:
      throw new Error("Unknown object name");
  }

  return node;
}

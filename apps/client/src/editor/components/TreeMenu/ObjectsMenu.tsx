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

import { addMesh } from "../../actions/AddMeshAction";
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
    // Create node
    const selectedId = createNode(name);

    // Select new node
    useEditorStore.setState({ selectedId });
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

  switch (name) {
    case ObjectName.Box: {
      const mesh = new BoxMesh();
      addMesh(mesh);

      node.meshId = mesh.id;
      node.collider = new BoxCollider();
      break;
    }

    case ObjectName.Sphere: {
      const mesh = new SphereMesh();
      addMesh(mesh);

      node.meshId = mesh.id;
      node.collider = new SphereCollider();
      break;
    }

    case ObjectName.Cylinder: {
      const mesh = new CylinderMesh();
      addMesh(mesh);

      node.meshId = mesh.id;
      node.collider = new CylinderCollider();
      break;
    }

    case ObjectName.glTF: {
      const mesh = new GLTFMesh();
      addMesh(mesh);

      node.meshId = mesh.id;
      node.collider = new MeshCollider();
      node.collider.meshId = mesh.id;
      break;
    }
  }

  addNode(node);

  return node.id;
}

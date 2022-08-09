import {
  BoxBufferGeometry,
  CylinderBufferGeometry,
  Mesh,
  MeshStandardMaterial,
  Object3D,
  SphereBufferGeometry,
} from "three";

import { TreeItem } from "@wired-xr/new-engine";

import { useStudioStore } from "../../store";
import { updateTree } from "../../utils/scene";

enum ObjectName {
  Box = "Box",
  Sphere = "Sphere",
  Cylinder = "Cylinder",
}

export default function ObjectsMenu() {
  function addObject(name: ObjectName) {
    const engine = useStudioStore.getState().engine;
    if (!engine) return;

    // Create three.js object
    const object = getObject(name);
    engine.renderThread.addObject(object);

    // Create tree item
    const treeItem = new TreeItem();
    treeItem.threeUUID = object.uuid;
    treeItem.name = object.name;
    engine.tree.addChild(treeItem);

    // Update tree
    updateTree();
  }

  return (
    <div className="p-2">
      {Object.values(ObjectName).map((name) => (
        <button
          key={name}
          onClick={() => addObject(name)}
          className="w-full flex hover:bg-primaryContainer hover:text-onPrimaryContainer
                     rounded px-4 py-1 transition items-center"
        >
          {name}
        </button>
      ))}
    </div>
  );
}

function getObject(name: ObjectName): Object3D {
  switch (name) {
    case ObjectName.Box:
      const boxGeometry = new BoxBufferGeometry(1, 1, 1);
      const boxMaterial = new MeshStandardMaterial({ color: 0xffff00 });
      const boxMesh = new Mesh(boxGeometry, boxMaterial);
      boxMesh.name = "Box";
      return boxMesh;
    case ObjectName.Sphere:
      const sphereGeometry = new SphereBufferGeometry(0.5);
      const sphereMaterial = new MeshStandardMaterial({ color: 0x00ff00 });
      const sphereMesh = new Mesh(sphereGeometry, sphereMaterial);
      sphereMesh.name = "Sphere";
      return sphereMesh;
    case ObjectName.Cylinder:
      const cylinderGeometry = new CylinderBufferGeometry(0.5, 0.5, 1);
      const cylinderMaterial = new MeshStandardMaterial({ color: 0x0000ff });
      const cylinderMesh = new Mesh(cylinderGeometry, cylinderMaterial);
      cylinderMesh.name = "Cylinder";
      return cylinderMesh;
  }
}

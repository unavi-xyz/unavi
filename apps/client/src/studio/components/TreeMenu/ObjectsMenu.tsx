import { UserData } from "@wired-xr/engine";

import { useStudioStore } from "../../store";
import { getObject, updateTree } from "../../utils/scene";

enum ObjectName {
  Box = "Box",
  Sphere = "Sphere",
  Cylinder = "Cylinder",
}

export default function ObjectsMenu() {
  function addObject(name: ObjectName) {
    const { engine, root } = useStudioStore.getState();
    if (!engine) return;

    // Create three.js object
    const object = createObject(name);
    // Get parent
    const selectedId = useStudioStore.getState().selectedId;
    const selected = selectedId ? getObject(selectedId) : undefined;
    const parent = selected ?? root;

    // Add to scene
    parent.add(object);

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

function createObject(name: ObjectName) {
  // switch (name) {
  //   case ObjectName.Box:
  //     // Mesh
  //     const boxGeometry = new BoxBufferGeometry(1, 1, 1);
  //     const boxMaterial = new MeshStandardMaterial({ color: 0xffff00 });
  //     const boxMesh = new Mesh(boxGeometry, boxMaterial);
  //     boxMesh.name = "Box";
  //     // Physics
  //     const boxUserData: UserData = {
  //       OMI_collider: {
  //         type: "box",
  //         extents: [1, 1, 1],
  //       },
  //       OMI_physics_body: {
  //         type: "static",
  //       },
  //     };
  //     boxMesh.userData = boxUserData;
  //     return boxMesh;
  //   case ObjectName.Sphere:
  //     // Mesh
  //     const sphereGeometry = new SphereBufferGeometry(0.5);
  //     const sphereMaterial = new MeshStandardMaterial({ color: 0x00ff00 });
  //     const sphereMesh = new Mesh(sphereGeometry, sphereMaterial);
  //     sphereMesh.name = "Sphere";
  //     // Physics
  //     const sphereUserData: UserData = {
  //       OMI_collider: {
  //         type: "sphere",
  //         radius: 0.5,
  //       },
  //       OMI_physics_body: {
  //         type: "static",
  //       },
  //     };
  //     sphereMesh.userData = sphereUserData;
  //     return sphereMesh;
  //   case ObjectName.Cylinder:
  //     const cylinderGeometry = new CylinderBufferGeometry(0.5, 0.5, 1);
  //     const cylinderMaterial = new MeshStandardMaterial({ color: 0x0000ff });
  //     const cylinderMesh = new Mesh(cylinderGeometry, cylinderMaterial);
  //     cylinderMesh.name = "Cylinder";
  //     return cylinderMesh;
  // }
}

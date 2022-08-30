import { addComponent, addEntity } from "bitecs";

import { Box, Cylinder, SceneObject, Sphere } from "@wired-xr/engine";

import { useStudioStore } from "../../store";
import { addObjectName } from "../../utils/names";
import { updateTree } from "../../utils/tree";

enum ObjectName {
  Box = "Box",
  Sphere = "Sphere",
  Cylinder = "Cylinder",
}

export default function ObjectsMenu() {
  function addObject(name: ObjectName) {
    // Create entity
    const eid = createObject(name);
    if (eid === undefined) return;

    // Update scene
    const engine = useStudioStore.getState().engine;
    engine?.updateScene();
    updateTree();
  }

  return (
    <div className="p-2 space-y-0.5">
      {Object.values(ObjectName).map((name) => (
        <button
          key={name}
          onClick={() => addObject(name)}
          className="w-full flex hover:bg-primaryContainer hover:text-onPrimaryContainer
                     rounded px-4 py-0.5 transition items-center"
        >
          {name}
        </button>
      ))}
    </div>
  );
}

function createObject(name: ObjectName) {
  const engine = useStudioStore.getState().engine;
  if (!engine) return;
  const world = engine.world;

  // Create entity
  const eid = addEntity(world);

  // Set name
  const nameId = addObjectName(name);

  // Add SceneObject component
  addComponent(world, SceneObject, eid);
  SceneObject.name[eid] = nameId;
  SceneObject.position.x[eid] = 0;
  SceneObject.position.y[eid] = 0;
  SceneObject.position.z[eid] = 0;
  SceneObject.rotation.x[eid] = 0;
  SceneObject.rotation.y[eid] = 0;
  SceneObject.rotation.z[eid] = 0;
  SceneObject.rotation.w[eid] = 1;
  SceneObject.scale.x[eid] = 1;
  SceneObject.scale.y[eid] = 1;
  SceneObject.scale.z[eid] = 1;

  // Add object component
  switch (name) {
    case ObjectName.Box:
      addComponent(world, Box, eid);
      Box.width[eid] = 1;
      Box.height[eid] = 1;
      Box.depth[eid] = 1;
      break;
    case ObjectName.Sphere:
      addComponent(world, Sphere, eid);
      Sphere.radius[eid] = 0.5;
      Sphere.widthSegments[eid] = 32;
      Sphere.heightSegments[eid] = 32;
      break;
    case ObjectName.Cylinder:
      addComponent(world, Cylinder, eid);
      Cylinder.radiusTop[eid] = 0.5;
      Cylinder.radiusBottom[eid] = 0.5;
      Cylinder.height[eid] = 1;
      Cylinder.radialSegments[eid] = 32;
      break;
    default:
      throw new Error("Unknown object name");
  }

  return eid;
}

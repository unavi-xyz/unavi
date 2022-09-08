import { Entity } from "@wired-labs/engine";

import { useStudioStore } from "../store";

export class SetGeometry {
  constructor(entity: Entity) {
    const { tree, engine } = useStudioStore.getState();

    // Set entity
    tree[entity.id] = entity;

    // Update tree
    useStudioStore.setState({ tree });

    // Get geometry
    let geometry: number[] = [];

    switch (entity.type) {
      case "Box":
        geometry = [entity.width, entity.height, entity.depth];
        break;
      case "Sphere":
        geometry = [entity.radius, entity.widthSegments, entity.heightSegments];
        break;
      case "Cylinder":
        geometry = [
          entity.radiusTop,
          entity.radiusBottom,
          entity.height,
          entity.radialSegments,
        ];
        break;
    }

    // Update engine
    if (engine) engine.renderThread.setGeometry(entity.id, geometry);
  }
}

export function setGeometry(entity: Entity) {
  return new SetGeometry(entity);
}

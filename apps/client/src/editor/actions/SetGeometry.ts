import { Entity } from "@wired-labs/engine";

import { useEditorStore } from "../store";

export class SetGeometry {
  constructor(entity: Entity) {
    const { scene, engine } = useEditorStore.getState();

    // Set entity
    scene.entities[entity.id] = entity;

    // Update scene
    useEditorStore.setState({ scene });

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

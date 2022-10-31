import {
  BoxGeometry,
  BufferGeometry,
  CylinderGeometry,
  SphereGeometry,
} from "three";

import { MeshJSON } from "../../../scene";
import { SceneMap } from "../types";
import { createAttribute } from "./createAttribute";

export function createMeshGeometry(json: MeshJSON, map: SceneMap) {
  function setMorphAttribute(
    geometry: BufferGeometry,
    threeName: string,
    accessorIds: string[]
  ) {
    if (accessorIds.length === 0) return;
    const attributes = accessorIds.map((id) => createAttribute(id, map));
    geometry.morphAttributes[threeName] = attributes;
  }

  function setAttribute(
    geometry: BufferGeometry,
    threeName: string,
    accessorId: string | null
  ) {
    if (accessorId === null) return;

    const attribute = createAttribute(accessorId, map);
    geometry.setAttribute(threeName, attribute);
  }

  switch (json.type) {
    case "Box": {
      return new BoxGeometry(json.width, json.height, json.depth);
    }

    case "Sphere": {
      return new SphereGeometry(
        json.radius,
        json.widthSegments,
        json.heightSegments
      );
    }

    case "Cylinder": {
      return new CylinderGeometry(
        json.radius,
        json.radius,
        json.height,
        json.radialSegments
      );
    }

    case "Primitives": {
      const geometries = json.primitives.map((primitive) => {
        const geometry = new BufferGeometry();
        geometry.morphTargetsRelative = true;

        // Set indices
        if (primitive.indicesId) {
          const attribute = createAttribute(primitive.indicesId, map);
          geometry.setIndex(attribute);
        }

        // Set attributes
        setAttribute(geometry, "position", primitive.POSITION);
        setAttribute(geometry, "normal", primitive.NORMAL);
        setAttribute(geometry, "uv", primitive.TEXCOORD_0);
        setAttribute(geometry, "uv2", primitive.TEXCOORD_1);
        setAttribute(geometry, "color", primitive.COLOR_0);
        setAttribute(geometry, "skinIndex", primitive.JOINTS_0);
        setAttribute(geometry, "skinWeight", primitive.WEIGHTS_0);

        // Set morph targets
        setMorphAttribute(geometry, "position", primitive.morphPositionIds);
        setMorphAttribute(geometry, "normal", primitive.morphNormalIds);
        setMorphAttribute(geometry, "tangent", primitive.morphTangentIds);

        return geometry;
      });

      return geometries;
    }
  }
}

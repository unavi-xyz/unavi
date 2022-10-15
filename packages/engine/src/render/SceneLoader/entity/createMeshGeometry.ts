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

    case "Primitive": {
      const geometry = new BufferGeometry();
      geometry.morphTargetsRelative = true;

      // Set indices
      if (json.indicesId) {
        const attribute = createAttribute(json.indicesId, map);
        geometry.setIndex(attribute);
      }

      // Set attributes
      setAttribute(geometry, "position", json.POSITION);
      setAttribute(geometry, "normal", json.NORMAL);
      setAttribute(geometry, "uv", json.TEXCOORD_0);
      setAttribute(geometry, "uv2", json.TEXCOORD_1);
      setAttribute(geometry, "color", json.COLOR_0);
      setAttribute(geometry, "skinIndex", json.JOINTS_0);
      setAttribute(geometry, "skinWeight", json.WEIGHTS_0);

      // Set morph targets
      setMorphAttribute(geometry, "position", json.morphPositionIds);
      setMorphAttribute(geometry, "normal", json.morphNormalIds);
      setMorphAttribute(geometry, "tangent", json.morphTangentIds);

      return geometry;
    }
  }
}

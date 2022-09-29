import {
  BoxBufferGeometry,
  BufferGeometry,
  CylinderBufferGeometry,
  SphereBufferGeometry,
} from "three";

import { MeshJSON } from "../../../scene";
import { RenderScene } from "../../RenderScene";
import { SceneMap } from "../types";
import { createAttribute } from "./createAttribute";

export function createMeshGeometry(
  json: MeshJSON,
  map: SceneMap,
  scene: RenderScene
) {
  function setMorphAttribute(
    geometry: BufferGeometry,
    threeName: string,
    accessorIds: string[]
  ) {
    if (accessorIds.length === 0) return;
    const attributes = accessorIds.map((id) => createAttribute(id, map, scene));
    geometry.morphAttributes[threeName] = attributes;
  }

  function setAttribute(
    geometry: BufferGeometry,
    threeName: string,
    accessorId: string | null
  ) {
    if (accessorId === null) return;

    const attribute = createAttribute(accessorId, map, scene);
    geometry.setAttribute(threeName, attribute);
  }

  switch (json.type) {
    case "Box":
      return new BoxBufferGeometry(json.width, json.height, json.depth);
    case "Sphere":
      return new SphereBufferGeometry(
        json.radius,
        json.widthSegments,
        json.heightSegments
      );
    case "Cylinder":
      return new CylinderBufferGeometry(
        json.radius,
        json.radius,
        json.height,
        json.radialSegments
      );
    case "Primitive":
      const primitiveGeometry = new BufferGeometry();
      primitiveGeometry.morphTargetsRelative = true;

      // Set indices
      if (json.indicesId) {
        const attribute = createAttribute(json.indicesId, map, scene);
        primitiveGeometry.setIndex(attribute);
      }

      // Set attributes
      setAttribute(primitiveGeometry, "position", json.POSITION);
      setAttribute(primitiveGeometry, "normal", json.NORMAL);
      setAttribute(primitiveGeometry, "uv", json.TEXCOORD_0);
      setAttribute(primitiveGeometry, "uv2", json.TEXCOORD_1);
      setAttribute(primitiveGeometry, "color", json.COLOR_0);
      setAttribute(primitiveGeometry, "skinIndex", json.JOINTS_0);
      setAttribute(primitiveGeometry, "skinWeight", json.WEIGHTS_0);

      // Set morph targets
      setMorphAttribute(primitiveGeometry, "position", json.morphPositionIds);
      setMorphAttribute(primitiveGeometry, "normal", json.morphNormalIds);
      setMorphAttribute(primitiveGeometry, "tangent", json.morphTangentIds);

      return primitiveGeometry;
  }
}

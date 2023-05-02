import {
  BoxGeometry,
  BufferGeometry,
  CylinderGeometry,
  GLBufferAttribute,
  SphereGeometry,
} from "three";

import { CustomMesh } from "../../../scene";
import { THREE_ATTRIBUTE_NAMES } from "../constants";

/**
 * Get a custom mesh's geometry data using Three.js.
 *
 * @param json The custom mesh JSON.
 * @returns The geometry data.
 */
export function getCustomMeshData(json: CustomMesh) {
  let geometry: BufferGeometry;

  switch (json.type) {
    case "Box": {
      const { width, height, depth } = json;
      geometry = new BoxGeometry(width, height, depth);
      break;
    }

    case "Sphere": {
      const { radius, widthSegments, heightSegments } = json;
      geometry = new SphereGeometry(radius, widthSegments, heightSegments);
      break;
    }

    case "Cylinder": {
      const { radiusTop, radiusBottom, height, radialSegments } = json;
      geometry = new CylinderGeometry(radiusTop, radiusBottom, height, radialSegments);
      break;
    }
  }

  const positions = geometry.getAttribute(THREE_ATTRIBUTE_NAMES.POSITION);
  const normals = geometry.getAttribute(THREE_ATTRIBUTE_NAMES.NORMAL);
  const indices = geometry.getIndex();

  if (!indices) throw new Error("No indices found");
  if (positions instanceof GLBufferAttribute)
    throw new Error("Positions are not a buffer attribute");
  if (normals instanceof GLBufferAttribute) throw new Error("Normals are not a buffer attribute");

  return { indices: indices.array, normals: normals.array, positions: positions.array };
}

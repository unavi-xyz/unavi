import { BoxGeometry, BufferGeometry, CylinderGeometry, SphereGeometry } from "three";

import { CustomMesh } from "../../scene";
import { THREE_ATTRIBUTE_NAMES } from "./constants";

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

  return { positions: positions.array, normals: normals.array, indices: indices.array };
}

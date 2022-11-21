import { Triplet } from "../../types";
import { BoxCollider } from "../collider/BoxCollider";
import { CylinderCollider } from "../collider/CylinderCollider";
import { MeshCollider } from "../collider/MeshCollider";
import { SphereCollider } from "../collider/SphereCollider";
import {
  BoxColliderJSON,
  CylinderColliderJSON,
  HullColliderJSON,
  MeshColliderJSON,
  SphereColliderJSON,
} from "../collider/types";
import { MeshJSON } from "../mesh/types";
import { NodeJSON } from "../types";

/*
 * Converts an AutoCollider to a standard collider type, depending on the node's mesh.
 */
export function convertAutoCollider(
  node: NodeJSON,
  mesh: MeshJSON | undefined,
  globalScale: Triplet
):
  | BoxColliderJSON
  | SphereColliderJSON
  | CylinderColliderJSON
  | MeshColliderJSON
  | HullColliderJSON
  | null {
  if (node.collider?.type === "auto") {
    if (!mesh) return null;

    // If non-uniform scale, use mesh collider
    const isUniformScale = globalScale.every((e) => e === globalScale[0]);
    if (!isUniformScale) {
      const meshCollider = new MeshCollider();
      meshCollider.meshId = node.meshId;
      return meshCollider;
    }

    // Otherwise, apply scale to shape collider
    const scale = globalScale[0];
    switch (mesh.type) {
      case "Box": {
        const boxCollider = new BoxCollider();
        boxCollider.size = [
          mesh.width * scale,
          mesh.height * scale,
          mesh.depth * scale,
        ];
        return boxCollider;
        break;
      }

      case "Sphere": {
        const sphereCollider = new SphereCollider();
        sphereCollider.radius = mesh.radius * scale;
        return sphereCollider;
        break;
      }

      case "Cylinder": {
        const cylinderCollider = new CylinderCollider();
        cylinderCollider.radius = mesh.radius * scale;
        cylinderCollider.height = mesh.height * scale;
        return cylinderCollider;
        break;
      }

      case "Primitives": {
        const meshCollider = new MeshCollider();
        meshCollider.meshId = node.meshId;
        return meshCollider;
      }

      default: {
        return null;
      }
    }
  }

  return node.collider;
}

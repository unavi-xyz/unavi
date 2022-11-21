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
  mesh: MeshJSON | undefined
):
  | BoxColliderJSON
  | SphereColliderJSON
  | CylinderColliderJSON
  | MeshColliderJSON
  | HullColliderJSON
  | null {
  if (node.collider?.type === "auto") {
    switch (mesh?.type) {
      case "Box": {
        const boxCollider = new BoxCollider();
        boxCollider.size = [mesh.width, mesh.height, mesh.depth];
        return boxCollider;
        break;
      }

      case "Sphere": {
        const sphereCollider = new SphereCollider();
        sphereCollider.radius = mesh.radius;
        return sphereCollider;
        break;
      }

      case "Cylinder": {
        const cylinderCollider = new CylinderCollider();
        cylinderCollider.radius = mesh.radius;
        cylinderCollider.height = mesh.height;
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

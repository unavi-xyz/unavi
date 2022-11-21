import {
  BoxGeometry,
  BufferGeometry,
  CylinderGeometry,
  Mesh,
  MeshBasicMaterial,
  SphereGeometry,
} from "three";
import { mergeBufferGeometries } from "three/examples/jsm/utils/BufferGeometryUtils";

import { calcGlobalScale } from "../../../scene/utils/calcGlobalScale";
import { convertAutoCollider } from "../../../scene/utils/convertAutoCollider";
import { PostMessage } from "../../../types";
import { FromRenderMessage } from "../../types";
import { SceneMap, UserData } from "../types";
import { removeCollider } from "./removeCollider";

const wireframeMaterial = new MeshBasicMaterial({
  color: 0x000000,
  wireframe: true,
});

export function createCollider(
  nodeId: string,
  map: SceneMap,
  postMessage: PostMessage<FromRenderMessage>
) {
  const node = map.nodes.get(nodeId);
  if (!node) throw new Error("Node not found");

  // Remove previous collider
  removeCollider(nodeId, map);

  // Create new collider
  const nodes = Array.from(map.nodes.values());
  const globalScale = calcGlobalScale(node, nodes);
  const isUniformScale = globalScale.every((e) => e === globalScale[0]);
  const nodeMesh = node.meshId ? map.meshes.get(node.meshId) : undefined;
  const nodeCollider = convertAutoCollider(node, nodeMesh, globalScale);

  let visual: Mesh | null = null;

  switch (nodeCollider?.type) {
    case "box": {
      visual = new Mesh(
        new BoxGeometry(...nodeCollider.size),
        wireframeMaterial
      );
      break;
    }

    case "sphere": {
      visual = new Mesh(
        new SphereGeometry(nodeCollider.radius),
        wireframeMaterial
      );
      break;
    }

    case "cylinder": {
      visual = new Mesh(
        new CylinderGeometry(
          nodeCollider.radius,
          nodeCollider.radius,
          nodeCollider.height,
          32
        ),
        wireframeMaterial
      );
      break;
    }

    case "hull":
    case "mesh": {
      const object = map.objects.get(nodeId);
      if (!object) break;

      if (!nodeCollider.meshId) break;
      const colliderMesh = map.meshes.get(nodeCollider.meshId);
      if (!colliderMesh) throw new Error("Collider mesh not found");

      // Get the visual mesh geometry
      const geometries: BufferGeometry[] = [];
      switch (colliderMesh.type) {
        case "Box": {
          geometries.push(
            new BoxGeometry(
              colliderMesh.width,
              colliderMesh.height,
              colliderMesh.depth
            )
          );
          break;
        }

        case "Sphere": {
          geometries.push(
            new SphereGeometry(
              colliderMesh.radius,
              colliderMesh.widthSegments,
              colliderMesh.heightSegments
            )
          );
          break;
        }

        case "Cylinder": {
          geometries.push(
            new CylinderGeometry(
              colliderMesh.radius,
              colliderMesh.radius,
              colliderMesh.height,
              colliderMesh.radialSegments
            )
          );
          break;
        }

        case "Primitives": {
          const globalGeometries: BufferGeometry[] = [];

          colliderMesh.primitives.forEach((primitive) => {
            const primitiveObject = map.objects.get(primitive.id);
            if (!primitiveObject) throw new Error("Primitive not found");

            primitiveObject.traverse((child) => {
              if (child instanceof Mesh) {
                const geometry = child.geometry as BufferGeometry;

                geometries.push(geometry);

                const cloned = geometry.clone();
                cloned.applyMatrix4(child.matrixWorld);
                globalGeometries.push(cloned);
              }
            });
          });

          if (globalGeometries.length === 0) break;

          // Merge global geometries
          const geometry = mergeBufferGeometries(globalGeometries);
          geometry.applyMatrix4(object.matrixWorld);

          // Remove indices if hull
          if (nodeCollider?.type === "hull") geometry.deleteAttribute("index");

          // Send geometry attributes to physics thread
          const attribute = geometry.getAttribute("position");
          const positions = Float32Array.from(attribute.array);
          const indices = geometry.index
            ? Uint32Array.from(geometry.index.array)
            : undefined;

          postMessage({
            subject: "set_collider_geometry",
            data: {
              nodeId,
              positions,
              indices,
            },
          });
          break;
        }
      }

      if (geometries.length === 0) break;

      // Merge geometries
      const geometry = mergeBufferGeometries(geometries);

      // Remove indices if hull
      if (nodeCollider.type === "hull") geometry.deleteAttribute("index");

      // Create mesh
      visual = new Mesh(geometry, wireframeMaterial);
      break;
    }
  }

  // Hack to fix scaling of visual after auto collider conversion
  if (isUniformScale) visual?.scale.divideScalar(globalScale[0]);

  if (visual) {
    const object = map.objects.get(nodeId);
    if (!object) throw new Error("Object not found");

    // Set visual
    visual.userData[UserData.isVisual] = true;
    map.colliders.set(nodeId, visual);

    // Add visual to scene
    object.add(visual);
  }
}

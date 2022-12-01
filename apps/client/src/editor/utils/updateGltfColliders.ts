import { AutoCollider, ColliderJSON, HullCollider, MeshCollider } from "engine";

import { useEditorStore } from "../store";

export function updateGltfColliders(nodeId: string) {
  const { engine } = useEditorStore.getState();
  if (!engine) return;

  const node = engine.scene.nodes[nodeId];
  if (!node) throw new Error(`Node ${nodeId} not found`);

  if (node.meshId) {
    const mesh = engine.scene.meshes[node.meshId];
    if (!mesh) throw new Error(`Mesh ${node.meshId} not found`);

    if (mesh.type === "glTF") {
      node.children.forEach((child) =>
        setGltfCollider(child.id, node.collider)
      );
    }
  }
}

function setGltfCollider(nodeId: string, rootCollider: ColliderJSON | null) {
  const { engine } = useEditorStore.getState();
  if (!engine) return;

  const node = engine.scene.nodes[nodeId];
  if (!node?.isInternal) return;

  switch (rootCollider?.type) {
    case "auto": {
      const collider = new AutoCollider();
      engine.scene.updateNode(nodeId, { collider: collider.toJSON() });
      break;
    }

    case "hull": {
      const collider = new HullCollider();
      collider.meshId = node.meshId;
      engine.scene.updateNode(nodeId, { collider: collider.toJSON() });
      break;
    }

    case "mesh": {
      const collider = new MeshCollider();
      collider.meshId = node.meshId;
      engine.scene.updateNode(nodeId, { collider: collider.toJSON() });
      break;
    }

    default: {
      engine.scene.updateNode(nodeId, { collider: null });
    }
  }

  // Repeat for children
  node.children.forEach((child) => setGltfCollider(child.id, rootCollider));
}

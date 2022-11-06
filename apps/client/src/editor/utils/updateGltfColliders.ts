import { HullCollider, MeshCollider } from "@wired-labs/engine";

import { useEditorStore } from "../store";

export function updateGltfColliders(nodeId: string) {
  const { engine } = useEditorStore.getState();
  if (!engine) return;

  const node = engine.scene.nodes[nodeId];

  if (node?.meshId) {
    const mesh = engine.scene.meshes[node.meshId];
    if (mesh?.type === "glTF") {
      node.children.forEach((child) => {
        if (!child.isInternal) return;

        switch (node.collider?.type) {
          case "hull": {
            const childCollider = new HullCollider();
            childCollider.meshId = child.meshId;
            engine.scene.updateNode(child.id, { collider: childCollider });
            break;
          }

          case "mesh": {
            const childCollider = new MeshCollider();
            childCollider.meshId = child.meshId;
            engine.scene.updateNode(child.id, { collider: childCollider });
            break;
          }

          default: {
            engine.scene.updateNode(child.id, { collider: null });
          }
        }
      });
    }
  }
}

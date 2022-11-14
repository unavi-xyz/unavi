import { GLTFMesh } from "@wired-labs/engine";
import { useState } from "react";

import Button from "../../../../ui/Button";
import ButtonFileInput from "../../../../ui/ButtonFileInput";
import Dialog from "../../../../ui/Dialog";
import { removeMesh } from "../../../actions/RemoveMeshAction";
import { updateMesh } from "../../../actions/UpdateMeshAction";
import { updateNode } from "../../../actions/UpdateNodeAction";
import { useSubscribeValue } from "../../../hooks/useSubscribeValue";
import { useEditorStore } from "../../../store";
import { updateGltfColliders } from "../../../utils/updateGltfColliders";
import { removeInternalFromNode } from "../utils/removeInternalFromNode";

interface Props {
  nodeId: string;
  mesh: GLTFMesh;
}

export default function GLTFMeshComponent({ nodeId, mesh }: Props) {
  const name = useSubscribeValue(mesh.name$);
  const uri = useSubscribeValue(mesh.uri$);

  const [openConfirmDialog, setOpenConfirmDialog] = useState(false);

  function handleSplit() {
    // Show node contents in editor
    removeInternalFromNode(nodeId);

    // Empty the node
    updateNode(nodeId, { meshId: null, collider: null });
    removeMesh(mesh.id);

    useEditorStore.setState({ changesToSave: true, selectedId: null });
    setOpenConfirmDialog(false);

    // Refresh selection to refresh tree menu
    setTimeout(() => useEditorStore.setState({ selectedId: nodeId }));
  }

  return (
    <>
      <Dialog
        open={openConfirmDialog}
        onClose={() => setOpenConfirmDialog(false)}
      >
        <div className="space-y-8">
          <h1 className="flex justify-center text-3xl font-bold">
            Confirm Unpack
          </h1>

          <div className="text-lg text-center">
            Unpacking a model will expose its internal meshes and materials to
            the editor. This action cannot be undone.
          </div>

          <div className="justify-end flex">
            <Button fullWidth variant="filled" onClick={handleSplit}>
              Confirm
            </Button>
          </div>
        </div>
      </Dialog>

      <div className="pt-1">
        <ButtonFileInput
          accept=".glb,.gltf"
          onChange={(e) => {
            if (!e.target.files) return;

            const file = e.target.files[0];
            if (!file) return;

            const isGlb = file.name.endsWith(".glb");
            const type = isGlb ? "model/gltf-binary" : "model/gltf+json";
            const blob = new Blob([file], { type });

            updateMesh(mesh.id, {
              name: file.name,
              uri: URL.createObjectURL(blob),
            });

            setTimeout(() => updateGltfColliders(nodeId), 5000);
          }}
        >
          {uri ? name : "Upload GLTF"}
        </ButtonFileInput>
      </div>

      {uri && (
        <Button
          fullWidth
          rounded="large"
          onClick={() => setOpenConfirmDialog(true)}
        >
          Unpack
        </Button>
      )}
    </>
  );
}

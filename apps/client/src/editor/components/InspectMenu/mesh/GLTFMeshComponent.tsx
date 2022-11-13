import { GLTFMesh } from "@wired-labs/engine";

import FileInput from "../../../../ui/FileInput";
import { updateMesh } from "../../../actions/UpdateMeshAction";
import { useSubscribeValue } from "../../../hooks/useSubscribeValue";
import { updateGltfColliders } from "../../../utils/updateGltfColliders";
import MenuRows from "../MenuRows";

interface Props {
  nodeId: string;
  mesh: GLTFMesh;
}

export default function GLTFMeshComponent({ nodeId, mesh }: Props) {
  const name = useSubscribeValue(mesh.name$);

  return (
    <MenuRows titles={["Model"]}>
      <FileInput
        displayName={name}
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

          setTimeout(() => {
            updateGltfColliders(nodeId);
          }, 1000);
        }}
      />
    </MenuRows>
  );
}

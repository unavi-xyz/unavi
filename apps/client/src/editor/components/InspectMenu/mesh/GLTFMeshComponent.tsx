import { GLTFMesh } from "@wired-labs/engine";

import FileInput from "../../../../ui/FileInput";
import { updateMesh } from "../../../actions/UpdateMeshAction";
import { useSubscribeValue } from "../../../hooks/useSubscribeValue";

interface Props {
  mesh: GLTFMesh;
}

export default function GLTFMeshComponent({ mesh }: Props) {
  const name = useSubscribeValue(mesh.name$);

  return (
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
      }}
    />
  );
}

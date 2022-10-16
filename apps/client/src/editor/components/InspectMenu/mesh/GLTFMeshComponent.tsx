import { GLTFMesh } from "@wired-labs/engine";

import FileInput from "../../../../ui/FileInput";
import { updateEntity } from "../../../actions/UpdateEntityAction";
import { useSubscribeValue } from "../../../hooks/useSubscribeValue";
import { useEditorStore } from "../../../store";
import ComponentMenu from "../ComponentMenu";

interface Props {
  entityId: string;
  mesh: GLTFMesh;
}

export default function GLTFMeshComponent({ entityId, mesh }: Props) {
  const name = useSubscribeValue(mesh.name$);

  return (
    <ComponentMenu title="Model">
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

          mesh.name = file.name;
          mesh.uri = URL.createObjectURL(blob);

          // This is a hack to force TransformControls to detach and reattach to the new object
          useEditorStore.setState({ selectedId: null });
          setTimeout(() => useEditorStore.setState({ selectedId: entityId }));

          updateEntity(entityId, { mesh: mesh.toJSON() });
        }}
      />
    </ComponentMenu>
  );
}

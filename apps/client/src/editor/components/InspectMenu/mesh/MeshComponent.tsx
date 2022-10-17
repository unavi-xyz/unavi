import {
  BoxMesh,
  CylinderMesh,
  GLTFMesh,
  Mesh,
  SphereMesh,
} from "@wired-labs/engine";

import { updateEntity } from "../../../actions/UpdateEntityAction";
import { useEditorStore } from "../../../store";
import SelectMenu from "../../ui/SelectMenu";
import ComponentMenu from "../ComponentMenu";
import MenuRows from "../MenuRows";
import BoxMeshComponent from "./BoxMeshComponent";
import CylinderMeshComponent from "./CylinderMeshComponent";
import GLTFMeshComponent from "./GLTFMeshComponent";
import SphereMeshComponent from "./SphereMeshComponent";

enum MeshType {
  Box = "Box",
  Sphere = "Sphere",
  Cylinder = "Cylinder",
  glTF = "glTF",
}

interface Props {
  entityId: string;
  mesh: Mesh;
}

export default function MeshComponent({ entityId, mesh }: Props) {
  return (
    <ComponentMenu
      title="Mesh"
      onRemove={() => {
        useEditorStore.setState({ selectedId: null });
        setTimeout(() => useEditorStore.setState({ selectedId: entityId }));

        updateEntity(entityId, { mesh: null });
      }}
    >
      <MenuRows titles={["Type"]}>
        <SelectMenu
          value={mesh.type}
          options={Object.values(MeshType)}
          onChange={(e) => {
            const type = e.target.value;

            useEditorStore.setState({ selectedId: null });
            setTimeout(() => useEditorStore.setState({ selectedId: entityId }));

            switch (type) {
              case MeshType.Box: {
                const mesh = new BoxMesh();
                updateEntity(entityId, { mesh: mesh.toJSON() });
                break;
              }

              case MeshType.Sphere: {
                const mesh = new SphereMesh();
                updateEntity(entityId, { mesh: mesh.toJSON() });
                break;
              }

              case MeshType.Cylinder: {
                const mesh = new CylinderMesh();
                updateEntity(entityId, { mesh: mesh.toJSON() });
                break;
              }

              case MeshType.glTF: {
                const mesh = new GLTFMesh();
                updateEntity(entityId, { mesh: mesh.toJSON() });
                break;
              }
            }
          }}
        />
      </MenuRows>

      {mesh.type === "Box" ? (
        <BoxMeshComponent entityId={entityId} mesh={mesh} />
      ) : mesh.type === "Sphere" ? (
        <SphereMeshComponent entityId={entityId} mesh={mesh} />
      ) : mesh.type === "Cylinder" ? (
        <CylinderMeshComponent entityId={entityId} mesh={mesh} />
      ) : mesh.type === "glTF" ? (
        <GLTFMeshComponent entityId={entityId} mesh={mesh} />
      ) : null}
    </ComponentMenu>
  );
}

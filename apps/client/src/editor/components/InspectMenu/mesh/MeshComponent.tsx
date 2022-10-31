import {
  BoxMesh,
  CylinderMesh,
  GLTFMesh,
  Mesh,
  SphereMesh,
} from "@wired-labs/engine";

import { updateNode } from "../../../actions/UpdateNodeAction";
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
  nodeId: string;
  mesh: Mesh;
}

export default function MeshComponent({ nodeId, mesh }: Props) {
  return (
    <ComponentMenu
      title="Mesh"
      onRemove={() => updateNode(nodeId, { mesh: null })}
    >
      <MenuRows titles={["Type"]}>
        <SelectMenu
          value={mesh.type}
          options={Object.values(MeshType)}
          onChange={(e) => {
            const type = e.target.value;

            switch (type) {
              case MeshType.Box: {
                const mesh = new BoxMesh();
                updateNode(nodeId, { mesh: mesh.toJSON() });
                break;
              }

              case MeshType.Sphere: {
                const mesh = new SphereMesh();
                updateNode(nodeId, { mesh: mesh.toJSON() });
                break;
              }

              case MeshType.Cylinder: {
                const mesh = new CylinderMesh();
                updateNode(nodeId, { mesh: mesh.toJSON() });
                break;
              }

              case MeshType.glTF: {
                const mesh = new GLTFMesh();
                updateNode(nodeId, { mesh: mesh.toJSON() });
                break;
              }
            }
          }}
        />
      </MenuRows>

      {mesh.type === "Box" ? (
        <BoxMeshComponent nodeId={nodeId} mesh={mesh} />
      ) : mesh.type === "Sphere" ? (
        <SphereMeshComponent nodeId={nodeId} mesh={mesh} />
      ) : mesh.type === "Cylinder" ? (
        <CylinderMeshComponent nodeId={nodeId} mesh={mesh} />
      ) : mesh.type === "glTF" ? (
        <GLTFMeshComponent nodeId={nodeId} mesh={mesh} />
      ) : null}
    </ComponentMenu>
  );
}

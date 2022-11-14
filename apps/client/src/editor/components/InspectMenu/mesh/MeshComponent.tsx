import {
  BoxMesh,
  CylinderMesh,
  GLTFMesh,
  Mesh,
  SphereMesh,
} from "@wired-labs/engine";

import { addMesh } from "../../../actions/AddMeshAction";
import { removeMesh } from "../../../actions/RemoveMeshAction";
import { removeNode } from "../../../actions/RemoveNodeAction";
import { updateNode } from "../../../actions/UpdateNodeAction";
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
  nodeId: string;
  mesh: Mesh;
}

export default function MeshComponent({ nodeId, mesh }: Props) {
  if (mesh.type === "Primitives") return null;

  return (
    <ComponentMenu
      title="Mesh"
      onRemove={() => {
        updateNode(nodeId, { meshId: null });

        if (mesh.type === "glTF") {
          const { engine } = useEditorStore.getState();
          if (!engine) throw new Error("Engine not found");

          const node = engine.scene.nodes[nodeId];
          if (!node) throw new Error("Node not found");

          // Remove internal children
          node.children.forEach((child) => {
            if (child.isInternal) {
              if (child.meshId) removeMesh(child.meshId);
              removeNode(child.id);
            }
          });
        }

        removeMesh(mesh.id);
      }}
    >
      <MenuRows titles={["Type"]}>
        <SelectMenu
          value={mesh.type}
          options={Object.values(MeshType)}
          onChange={(e) => {
            const type = e.target.value;

            switch (type) {
              case MeshType.Box: {
                const newMesh = new BoxMesh();
                addMesh(newMesh);
                updateNode(nodeId, { meshId: newMesh.id });
                removeMesh(mesh.id);
                break;
              }

              case MeshType.Sphere: {
                const newMesh = new SphereMesh();
                addMesh(newMesh);
                updateNode(nodeId, { meshId: newMesh.id });
                removeMesh(mesh.id);
                break;
              }

              case MeshType.Cylinder: {
                const newMesh = new CylinderMesh();
                addMesh(newMesh);
                updateNode(nodeId, { meshId: newMesh.id });
                removeMesh(mesh.id);
                break;
              }

              case MeshType.glTF: {
                const newMesh = new GLTFMesh();
                addMesh(newMesh);
                updateNode(nodeId, { meshId: newMesh.id });
                removeMesh(mesh.id);
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

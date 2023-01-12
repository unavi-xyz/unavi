import { MeshExtras } from "engine";

import { useMesh } from "../../../hooks/useMesh";
import { useMeshAttribute } from "../../../hooks/useMeshAttribute";
import SelectMenu from "../../ui/SelectMenu";
import ComponentMenu from "../ComponentMenu";
import MenuRows from "../MenuRows";
import BoxMeshComponent from "./BoxMeshComponent";
import CylinderMeshComponent from "./CylinderMeshComponent";
import PrimitiveComponent from "./PrimitiveComponent";
import SphereMeshComponent from "./SphereMeshComponent";

const MESH_TYPE = {
  Box: "Box",
  Sphere: "Sphere",
  Cylinder: "Cylinder",
  Model: "Model",
  Primitives: "Primitives",
} as const;

type MeshType = (typeof MESH_TYPE)[keyof typeof MESH_TYPE];

interface Props {
  meshId: string;
}

export default function MeshComponent({ meshId }: Props) {
  const mesh = useMesh(meshId);
  const extras = useMeshAttribute(meshId, "extras");
  const primitiveIds = useMeshAttribute(meshId, "primitives");

  if (!mesh) return null;

  const meshType: MeshType = extras?.customMesh?.type ?? MESH_TYPE.Primitives;

  return (
    <ComponentMenu
      title="Mesh"
      // onRemove={() => {
      //   updateNode(nodeId, { meshId: null });

      //   if (mesh.type === "glTF") {
      //     const { engine } = useEditorStore.getState();
      //     if (!engine) throw new Error("Engine not found");

      //     const node = engine.scene.nodes[nodeId];
      //     if (!node) throw new Error("Node not found");

      //     // Remove internal children
      //     node.children.forEach((child) => {
      //       if (child.isInternal) {
      //         if (child.meshId) removeMesh(child.meshId);
      //         removeNode(child.id);
      //       }
      //     });
      //   }

      //   removeMesh(mesh.id);
      // }}
    >
      <MenuRows titles={["Type"]}>
        <SelectMenu
          value={meshType}
          options={Object.values(MESH_TYPE)}
          onChange={(e) => {
            const type = e.target.value;

            if (type === MESH_TYPE.Primitives) {
              const newExtras: MeshExtras = {
                ...extras,
                customMesh: undefined,
              };

              mesh.setExtras(newExtras);
              return;
            }

            // Remove all primitives
            mesh.listPrimitives().forEach((primitive) => mesh.removePrimitive(primitive));

            // Set custom mesh
            switch (type) {
              case MESH_TYPE.Box: {
                const newExtras: MeshExtras = {
                  ...extras,
                  customMesh: {
                    type: "Box",
                    width: 1,
                    height: 1,
                    depth: 1,
                  },
                };

                mesh.setExtras(newExtras);
                break;
              }

              case MESH_TYPE.Sphere: {
                const newExtras: MeshExtras = {
                  ...extras,
                  customMesh: {
                    type: "Sphere",
                    radius: 0.5,
                    widthSegments: 32,
                    heightSegments: 32,
                  },
                };

                mesh.setExtras(newExtras);
                break;
              }

              case MESH_TYPE.Cylinder: {
                const newExtras: MeshExtras = {
                  ...extras,
                  customMesh: {
                    type: "Cylinder",
                    radiusTop: 0.5,
                    radiusBottom: 0.5,
                    height: 1,
                    radialSegments: 32,
                  },
                };

                mesh.setExtras(newExtras);
                break;
              }

              case MESH_TYPE.Model: {
                const newExtras: MeshExtras = {
                  ...extras,
                  customMesh: {
                    type: "Model",
                    uri: "",
                  },
                };

                mesh.setExtras(newExtras);
                break;
              }
            }
          }}
        />
      </MenuRows>

      <div className="pb-4" />

      {meshType === MESH_TYPE.Box ? (
        <BoxMeshComponent meshId={meshId} />
      ) : meshType === MESH_TYPE.Sphere ? (
        <SphereMeshComponent meshId={meshId} />
      ) : meshType === MESH_TYPE.Cylinder ? (
        <CylinderMeshComponent meshId={meshId} />
      ) : (
        primitiveIds?.map((primitiveId) => (
          <PrimitiveComponent key={primitiveId} primitiveId={primitiveId} />
        ))
      )}
    </ComponentMenu>
  );
}

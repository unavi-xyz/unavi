import { Mesh, Node } from "@gltf-transform/core";
import { MeshExtras } from "engine";

import { useMeshExtras } from "../../../hooks/useMeshExtras";
import { useStudio } from "../../Studio";
import SelectMenu from "../../ui/SelectMenu";
import ComponentMenu from "../ComponentMenu";
import MenuRows from "../ui/MenuRows";
import BoxMeshComponent from "./BoxMeshComponent";
import CylinderMeshComponent from "./CylinderMeshComponent";
import SphereMeshComponent from "./SphereMeshComponent";

const MESH_TYPE = {
  Box: "Box",
  Sphere: "Sphere",
  Cylinder: "Cylinder",
  Primitives: "Primitives",
} as const;

type MeshType = (typeof MESH_TYPE)[keyof typeof MESH_TYPE];

interface Props {
  mesh: Mesh;
}

export default function MeshComponent({ mesh }: Props) {
  const { engine } = useStudio();
  const extras = useMeshExtras(mesh);

  if (!mesh) return null;

  const meshType: MeshType = extras?.customMesh?.type ?? MESH_TYPE.Primitives;

  if (meshType === MESH_TYPE.Primitives)
    return (
      <div className="group space-y-4 rounded-2xl p-4 ring-neutral-300 transition">
        <div className="-mt-1 flex justify-between text-xl font-bold">
          <div>Mesh</div>
        </div>

        <MenuRows titles={["Type"]}>
          <div className="w-full cursor-default select-none rounded border border-neutral-300 bg-neutral-50 pl-1.5">
            Primitives
          </div>
        </MenuRows>
      </div>
    );

  return (
    <ComponentMenu
      title="Mesh"
      onRemove={() => {
        if (!engine) return;
        mesh.dispose();
      }}
    >
      <MenuRows titles={["Type"]}>
        <SelectMenu
          value={meshType}
          options={Object.values(MESH_TYPE).filter((type) => type !== MESH_TYPE.Primitives)}
          onChange={(e) => {
            const type = e.target.value;

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
            }

            // Force node refresh
            mesh.listParents().forEach((p) => {
              if (p instanceof Node) {
                p.setMesh(mesh);
              }
            });
          }}
        />
      </MenuRows>

      {meshType === MESH_TYPE.Box ? (
        <BoxMeshComponent mesh={mesh} />
      ) : meshType === MESH_TYPE.Sphere ? (
        <SphereMeshComponent mesh={mesh} />
      ) : meshType === MESH_TYPE.Cylinder ? (
        <CylinderMeshComponent mesh={mesh} />
      ) : null}
    </ComponentMenu>
  );
}

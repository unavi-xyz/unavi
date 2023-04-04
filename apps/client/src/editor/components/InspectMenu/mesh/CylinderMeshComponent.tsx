import { Mesh } from "@gltf-transform/core";

import { useMeshExtras } from "../../../hooks/useMeshExtras";
import EditorInput from "../../ui/EditorInput";
import MenuRows from "../ui/MenuRows";

interface Props {
  mesh: Mesh;
}

export default function CylinderMeshComponent({ mesh }: Props) {
  const extras = useMeshExtras(mesh);

  if (!mesh || extras?.customMesh?.type !== "Cylinder") return null;

  const { height, radialSegments, radiusBottom, radiusTop } = extras.customMesh;

  return (
    <>
      <MenuRows titles={["Height", "Radius Top", "Radius Buttom", "Radial Segments"]}>
        {[height, radiusTop, radiusBottom, radialSegments].map((value, i) => {
          const property =
            i === 0
              ? "height"
              : i === 1
              ? "radiusTop"
              : i === 2
              ? "radiusBottom"
              : "radialSegments";
          const name = ["Height", "Radius Top", "Radius Buttom", "Radial Segments"][i];
          const step = name === "Radial Segments" ? 1 : 0.1;

          return (
            <EditorInput
              key={name}
              name={name}
              type="number"
              value={value ?? 0}
              step={step}
              onChange={(e) => {
                if (extras?.customMesh?.type !== "Cylinder") return;

                const value = e.target.value;
                if (!value) return;

                const num = parseFloat(value);
                const rounded = Math.round(num * 1000) / 1000;

                extras.customMesh[property] = rounded;

                mesh.setExtras({ ...extras });
              }}
            />
          );
        })}
      </MenuRows>

      {/* <MaterialComponent meshId={meshId} /> */}
    </>
  );
}

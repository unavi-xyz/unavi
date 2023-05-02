import { Mesh } from "@gltf-transform/core";

import { useMeshExtras } from "../../../hooks/useMeshExtras";
import StudioInput from "../../ui/StudioInput";
import MenuRows from "../ui/MenuRows";

interface Props {
  mesh: Mesh;
}

export default function SphereMeshComponent({ mesh }: Props) {
  const extras = useMeshExtras(mesh);

  if (!mesh || extras?.customMesh?.type !== "Sphere") return null;

  const { radius, widthSegments, heightSegments } = extras.customMesh;

  return (
    <>
      <MenuRows titles={["Radius", "Width Segments", "Height Segments"]}>
        {[radius, widthSegments, heightSegments].map((value, i) => {
          const property = i === 0 ? "radius" : i === 1 ? "widthSegments" : "heightSegments";
          const name = ["Radius", "Width Segments", "Height Segments"][i];
          const step = name === "Radius" ? 0.1 : 1;

          return (
            <StudioInput
              key={name}
              name={name}
              type="number"
              value={value ?? 0}
              step={step}
              onChange={(e) => {
                if (extras?.customMesh?.type !== "Sphere") return;

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

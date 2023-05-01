import { Mesh } from "@gltf-transform/core";

import { useMeshExtras } from "../../../hooks/useMeshExtras";
import StudioInput from "../../ui/StudioInput";
import MenuRows from "../ui/MenuRows";

interface Props {
  mesh: Mesh;
}

export default function BoxMeshComponent({ mesh }: Props) {
  const extras = useMeshExtras(mesh);

  if (!mesh || extras?.customMesh?.type !== "Box") return null;

  const { width, height, depth } = extras.customMesh;

  return (
    <MenuRows titles={["Width", "Height", "Depth"]}>
      {[width, height, depth].map((value, i) => {
        const property = i === 0 ? "width" : i === 1 ? "height" : "depth";
        const name = ["Width", "Height", "Depth"][i];

        return (
          <StudioInput
            key={name}
            name={name}
            type="number"
            value={value ?? 0}
            step={0.1}
            onChange={(e) => {
              if (extras?.customMesh?.type !== "Box") return;

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
  );
}

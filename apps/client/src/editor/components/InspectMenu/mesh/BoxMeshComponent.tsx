import { useMesh } from "../../../hooks/useMesh";
import { useMeshAttribute } from "../../../hooks/useMeshAttribute";
import NumberInput from "../../ui/NumberInput";
import MenuRows from "../MenuRows";

interface Props {
  meshId: string;
}

export default function BoxMeshComponent({ meshId }: Props) {
  const mesh = useMesh(meshId);
  const extras = useMeshAttribute(meshId, "extras");

  if (!mesh || extras?.customMesh?.type !== "Box") return null;

  const { width, height, depth } = extras.customMesh;

  return (
    <>
      <MenuRows titles={["Width", "Height", "Depth"]}>
        {[width, height, depth].map((value, i) => {
          const property = i === 0 ? "width" : i === 1 ? "height" : "depth";
          const name = ["Width", "Height", "Depth"][i];

          return (
            <NumberInput
              key={name}
              name={name}
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

      {/* <MaterialComponent meshId={meshId} /> */}
    </>
  );
}

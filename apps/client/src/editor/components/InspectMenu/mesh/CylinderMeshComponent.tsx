import { useMesh } from "../../../hooks/useMesh";
import { useMeshAttribute } from "../../../hooks/useMeshAttribute";
import NumberInput from "../../ui/NumberInput";
import MenuRows from "../ui/MenuRows";

interface Props {
  meshId: string;
}

export default function CylinderMeshComponent({ meshId }: Props) {
  const mesh = useMesh(meshId);
  const extras = useMeshAttribute(meshId, "extras");

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
            <NumberInput
              key={name}
              name={name}
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

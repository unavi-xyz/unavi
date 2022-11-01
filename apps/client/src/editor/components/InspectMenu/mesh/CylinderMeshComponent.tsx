import { CylinderMesh } from "@wired-labs/engine";

import { updateMesh } from "../../../actions/UpdateMeshAction";
import { useSubscribeValue } from "../../../hooks/useSubscribeValue";
import NumberInput from "../../ui/NumberInput";
import MaterialComponent from "../MaterialComponent";
import MenuRows from "../MenuRows";

interface Props {
  nodeId: string;
  mesh: CylinderMesh;
}

export default function CylinderMeshComponent({ nodeId, mesh }: Props) {
  const radius = useSubscribeValue(mesh.radius$);
  const height = useSubscribeValue(mesh.height$);
  const radialSegments = useSubscribeValue(mesh.radialSegments$);

  return (
    <>
      <MenuRows titles={["Radius", "Height", "Radial Segments"]}>
        {[radius, height, radialSegments].map((value, i) => {
          const property =
            i === 0 ? "radius" : i === 1 ? "height" : "radialSegments";
          const name = ["Radius", "Height", "Radial Segments"][i];
          const step = name === "Radial Segments" ? 1 : 0.1;

          return (
            <NumberInput
              key={name}
              name={name}
              value={value ?? 0}
              step={step}
              onChange={(e) => {
                const value = e.target.value;
                if (!value) return;

                const num = parseFloat(value);
                const rounded = Math.round(num * 1000) / 1000;

                updateMesh(mesh.id, { [property]: rounded });
              }}
            />
          );
        })}
      </MenuRows>

      <MaterialComponent nodeId={nodeId} />
    </>
  );
}

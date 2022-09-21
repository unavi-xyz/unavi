import { CylinderMesh } from "@wired-labs/engine";

import { setGeometry } from "../../actions/SetGeometryAction";
import { updateEntity } from "../../actions/UpdateEntityAction";
import { useSubscribeValue } from "../../hooks/useSubscribeValue";
import { useEditorStore } from "../../store";
import NumberInput from "../ui/NumberInput";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./MenuRows";

interface Props {
  entityId: string;
  mesh: CylinderMesh;
}

export default function CylinderComponent({ entityId, mesh }: Props) {
  const radius = useSubscribeValue(mesh.radius$, 0.5);
  const height = useSubscribeValue(mesh.height$, 1);
  const radialSegments = useSubscribeValue(mesh.radialSegments$, 32);

  return (
    <ComponentMenu title="Geometry">
      <MenuRows
        titles={["Radius Top", "Radius Bottom", "Height", "Radial Segments"]}
      >
        {[radius, height, radialSegments].map((value, i) => {
          const property =
            i === 0 ? "radius" : i === 1 ? "height" : "radialSegments";
          const name = ["Radius", "Height", "Radial Segments"][i];
          const step = name === "Radial Segments" ? 1 : 0.1;

          return (
            <NumberInput
              key={name}
              name={name}
              value={value}
              step={step}
              onChange={(e) => {
                const value = e.target.value;
                if (!value) return;

                const num = parseFloat(value);
                const rounded = Math.round(num * 1000) / 1000;

                mesh[property] = rounded;

                updateEntity(entityId, { mesh });
              }}
            />
          );
        })}
      </MenuRows>
    </ComponentMenu>
  );
}

import { Cylinder } from "@wired-labs/engine";

import { setGeometry } from "../../actions/SetGeometry";
import { useEditorStore } from "../../store";
import NumberInput from "../ui/NumberInput";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./MenuRows";

interface Props {
  entityId: string;
}

export default function CylinderComponent({ entityId }: Props) {
  const radiusTop = useEditorStore((state) => {
    const entity = state.scene.entities[entityId] as Cylinder;
    return entity.radiusTop;
  });
  const radiusBottom = useEditorStore((state) => {
    const entity = state.scene.entities[entityId] as Cylinder;
    return entity.radiusBottom;
  });
  const height = useEditorStore((state) => {
    const entity = state.scene.entities[entityId] as Cylinder;
    return entity.height;
  });
  const radialSegments = useEditorStore((state) => {
    const entity = state.scene.entities[entityId] as Cylinder;
    return entity.radialSegments;
  });

  return (
    <ComponentMenu title="Geometry">
      <MenuRows
        titles={["Radius Top", "Radius Bottom", "Height", "Radial Segments"]}
      >
        {[radiusTop, radiusBottom, height, radialSegments].map((value, i) => {
          const property =
            i === 0
              ? "radiusTop"
              : i === 1
              ? "radiusBottom"
              : i === 2
              ? "height"
              : "radialSegments";
          const name = [
            "Radius Top",
            "Radius Bottom",
            "Height",
            "Radial Segments",
          ][i];
          const step = name === "Radial Segments" ? 1 : 0.1;

          return (
            <NumberInput
              key={name}
              name={name}
              value={value}
              step={step}
              onChange={(e) => {
                // @ts-ignore
                const value: string | null = e.target.value || null;
                if (value === null) return;

                const num = parseFloat(value);
                const rounded = Math.round(num * 1000) / 1000;

                const { scene } = useEditorStore.getState();
                const entity = scene.entities[entityId] as Cylinder;
                entity[property] = rounded;

                setGeometry(entity);
              }}
            />
          );
        })}
      </MenuRows>
    </ComponentMenu>
  );
}

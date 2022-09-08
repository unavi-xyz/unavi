import { Cylinder } from "@wired-labs/engine";

import NumberInput from "../../../ui/base/NumberInput";
import { setGeometry } from "../../actions/SetGeometry";
import { useEditorStore } from "../../store";
import ComponentMenu from "./ComponentMenu";

interface Props {
  entityId: string;
}

export default function CylinderComponent({ entityId }: Props) {
  const radiusTop = useEditorStore((state) => {
    const entity = state.tree[entityId] as Cylinder;
    return entity.radiusTop;
  });
  const radiusBottom = useEditorStore((state) => {
    const entity = state.tree[entityId] as Cylinder;
    return entity.radiusBottom;
  });
  const height = useEditorStore((state) => {
    const entity = state.tree[entityId] as Cylinder;
    return entity.height;
  });
  const radialSegments = useEditorStore((state) => {
    const entity = state.tree[entityId] as Cylinder;
    return entity.radialSegments;
  });

  return (
    <ComponentMenu title="Geometry">
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
          <div key={name} className="grid grid-cols-2 gap-3">
            <label htmlFor={name}>{name}</label>
            <div className="flex">
              <NumberInput
                name={name}
                value={value}
                step={step}
                onChange={(e) => {
                  // @ts-ignore
                  const value: string | null = e.target.value || null;
                  if (value === null) return;

                  const num = parseFloat(value);
                  const rounded = Math.round(num * 1000) / 1000;

                  const { tree } = useEditorStore.getState();
                  const entity = tree[entityId] as Cylinder;
                  entity[property] = rounded;

                  setGeometry(entity);
                }}
              />
            </div>
          </div>
        );
      })}
    </ComponentMenu>
  );
}

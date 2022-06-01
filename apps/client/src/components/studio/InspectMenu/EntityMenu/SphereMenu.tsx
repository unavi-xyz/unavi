import { Entity } from "@wired-xr/scene";

import { round } from "../../../../helpers/utils/round";
import MenuRow from "../MenuRow";
import NumberInput from "../NumberInput";

interface Props {
  selected: Entity<"Sphere">;
  handleChange: (key: string, value: any) => void;
}

export default function SphereMenu({ selected, handleChange }: Props) {
  const radius = selected.props.radius ?? 0.5;
  const widthSegments = selected.props.widthSegments ?? 16;
  const heightSegments = selected.props.heightSegments ?? 16;

  return (
    <>
      <MenuRow title="Radius">
        <NumberInput
          updatedValue={String(round(radius))}
          onChange={(e) =>
            handleChange("radius", round(Number(e.currentTarget.value)))
          }
        />
      </MenuRow>

      <MenuRow title="Width Segments">
        <NumberInput
          updatedValue={String(round(widthSegments))}
          onChange={(e) =>
            handleChange("widthSegments", round(Number(e.currentTarget.value)))
          }
        />
      </MenuRow>

      <MenuRow title="Height Segments">
        <NumberInput
          updatedValue={String(round(heightSegments))}
          onChange={(e) =>
            handleChange("heightSegments", round(Number(e.currentTarget.value)))
          }
        />
      </MenuRow>
    </>
  );
}

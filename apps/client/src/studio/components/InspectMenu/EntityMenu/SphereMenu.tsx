import { IEntity } from "@wired-xr/engine";

import { round } from "../../../../utils/round";
import MenuRow from "../MenuRow";
import NumberInput from "../NumberInput";

interface Props {
  selected: IEntity<"Sphere">;
  handleChange: (key: string, value: any) => void;
}

export default function SphereMenu({ selected, handleChange }: Props) {
  const radius = selected.props.radius ?? 0.5;
  const widthSegments = selected.props.widthSegments ?? 16;
  const heightSegments = selected.props.heightSegments ?? 16;
  const physics = selected.props.physics ?? true;

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

      <MenuRow title="Physics">
        <input
          type="checkbox"
          checked={physics}
          onChange={(e) => {
            handleChange("physics", e.target.checked);
          }}
        />
      </MenuRow>
    </>
  );
}

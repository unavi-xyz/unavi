import { Entity } from "@wired-xr/engine";

import { round } from "../../../../helpers/utils/round";
import MenuRow from "../MenuRow";
import NumberInput from "../NumberInput";

interface Props {
  selected: Entity<"Box">;
  handleChange: (key: string, value: any) => void;
}

export default function BoxMenu({ selected, handleChange }: Props) {
  const width = selected.props.width ?? 1;
  const height = selected.props.height ?? 1;
  const depth = selected.props.depth ?? 1;
  const physics = selected.props.physics ?? true;

  return (
    <>
      <MenuRow title="Width">
        <NumberInput
          updatedValue={String(round(width))}
          onChange={(e) =>
            handleChange("width", round(Number(e.currentTarget.value)))
          }
        />
      </MenuRow>

      <MenuRow title="Height">
        <NumberInput
          updatedValue={String(round(height))}
          onChange={(e) =>
            handleChange("height", round(Number(e.currentTarget.value)))
          }
        />
      </MenuRow>

      <MenuRow title="Depth">
        <NumberInput
          updatedValue={String(round(depth))}
          onChange={(e) =>
            handleChange("depth", round(Number(e.currentTarget.value)))
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

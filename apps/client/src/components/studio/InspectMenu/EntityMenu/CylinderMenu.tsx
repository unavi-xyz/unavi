import { Entity } from "@wired-xr/scene";

import { round } from "../../../../helpers/utils/round";
import MenuRow from "../MenuRow";
import NumberInput from "../NumberInput";

interface Props {
  selected: Entity<"Cylinder">;
  handleChange: (key: string, value: any) => void;
}

export default function CylinderMenu({ selected, handleChange }: Props) {
  const radiusTop = selected.props.radiusTop ?? 1;
  const radiusBottom = selected.props.radiusBottom ?? 1;
  const height = selected.props.height ?? 1;
  const radialSegments = selected.props.radialSegments ?? 8;
  const openEnded = selected.props.openEnded ?? false;

  return (
    <>
      <MenuRow title="Radius Top">
        <NumberInput
          updatedValue={String(round(radiusTop))}
          onChange={(e) =>
            handleChange("radiusTop", round(Number(e.currentTarget.value)))
          }
        />
      </MenuRow>

      <MenuRow title="Radius Bottom">
        <NumberInput
          updatedValue={String(round(radiusBottom))}
          onChange={(e) =>
            handleChange("radiusBottom", round(Number(e.currentTarget.value)))
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

      <MenuRow title="Radial Segments">
        <NumberInput
          updatedValue={String(round(radialSegments))}
          onChange={(e) =>
            handleChange(
              "radialSegments",
              Math.max(round(Number(e.currentTarget.value)), 2)
            )
          }
        />
      </MenuRow>

      <MenuRow title="Open Ended">
        <input
          type="checkbox"
          checked={openEnded}
          onChange={(e) => handleChange("openEnded", e.currentTarget.checked)}
        />
      </MenuRow>
    </>
  );
}

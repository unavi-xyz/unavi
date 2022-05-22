import { useAtomValue } from "jotai";

import { selectedAtom } from "../../../helpers/studio/atoms";
import { useStudioStore } from "../../../helpers/studio/store";
import CollapseMenu from "../../base/CollapseMenu";
import Geometry from "./Geometry";
import Transform from "./Transform";

export default function InspectMenu() {
  const selected = useAtomValue(selectedAtom);
  const closedInspectMenus = useStudioStore(
    (state) => state.closedInspectMenus
  );

  if (!selected) return null;

  return (
    <div className="p-4 space-y-2 w-full">
      <div className="flex justify-center text-xl font-bold">
        {selected.name}
      </div>

      <CollapseMenu
        open={!closedInspectMenus.includes("Transform")}
        toggle={() =>
          useStudioStore.getState().toggleClosedInspectMenu("Transform")
        }
        title="Transform"
      >
        <Transform />
      </CollapseMenu>

      <CollapseMenu
        open={!closedInspectMenus.includes("Geometry")}
        toggle={() =>
          useStudioStore.getState().toggleClosedInspectMenu("Geometry")
        }
        title="Geometry"
      >
        <Geometry />
      </CollapseMenu>
    </div>
  );
}

import { useAtomValue } from "jotai";

import { selectedAtom } from "../../../helpers/studio/atoms";
import { useStudioStore } from "../../../helpers/studio/store";
import CollapseMenu from "../../base/CollapseMenu";
import GeometryMenu from "./menus/GeometryMenu";
import MaterialMenu from "./menus/MaterialMenu";
import TransformMenu from "./menus/TransformMenu";

export default function InspectMenu() {
  const selected = useAtomValue(selectedAtom);
  const closedInspectMenus = useStudioStore(
    (state) => state.closedInspectMenus
  );

  if (!selected) return null;

  return (
    <div className="p-4 space-y-8 w-full">
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
        <TransformMenu />
      </CollapseMenu>

      <CollapseMenu
        open={!closedInspectMenus.includes("Geometry")}
        toggle={() =>
          useStudioStore.getState().toggleClosedInspectMenu("Geometry")
        }
        title="Geometry"
      >
        <GeometryMenu />
      </CollapseMenu>

      <CollapseMenu
        open={!closedInspectMenus.includes("Material")}
        toggle={() =>
          useStudioStore.getState().toggleClosedInspectMenu("Material")
        }
        title="Material"
      >
        <MaterialMenu />
      </CollapseMenu>
    </div>
  );
}

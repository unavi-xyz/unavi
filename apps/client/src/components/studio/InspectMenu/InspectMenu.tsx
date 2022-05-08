import { useAtomValue } from "jotai";

import { selectedAtom } from "../../../helpers/studio/atoms";
import CollapseMenu from "../../base/CollapseMenu";
import Geometry from "./Geometry";

import Transform from "./Transform";

export default function InspectMenu() {
  const selected = useAtomValue(selectedAtom);

  if (!selected) return null;

  return (
    <div className="p-4 space-y-2 w-full">
      <div className="flex justify-center text-xl font-bold">
        {selected.name}
      </div>

      <CollapseMenu title="Transform">
        <Transform />
      </CollapseMenu>

      <CollapseMenu title="Geometry">
        <Geometry />
      </CollapseMenu>
    </div>
  );
}

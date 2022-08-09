import { useStudioStore } from "../../../studio/store";
import LightsButton from "./LightsButton";
import ObjectsButton from "./ObjectsButton";
import SpecialsButton from "./SpecialsButton";
import TreeMenuItem from "./TreeMenuItem";

export default function TreeMenu() {
  const engine = useStudioStore((state) => state.engine);

  // Use treeNonce to force re-rendering of the tree menu
  useStudioStore((state) => state.treeNonce);

  if (!engine) return null;

  return (
    <div className="h-full flex flex-col">
      <div className="py-2 h-14 flex items-center justify-center space-x-2">
        <ObjectsButton />
        {/* <LightsButton />
        <SpecialsButton /> */}
      </div>

      <div className="pt-2 pr-4 w-full h-full overflow-y-auto">
        <TreeMenuItem item={engine.tree} isRoot />
      </div>
    </div>
  );
}

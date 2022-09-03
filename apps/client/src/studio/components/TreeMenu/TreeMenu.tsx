import { useStudioStore } from "../../../studio/store";
import ObjectsButton from "./ObjectsButton";
import TreeMenuItem from "./TreeMenuItem";

export default function TreeMenu() {
  // Use treeNonce to force re-rendering of the tree menu
  useStudioStore((state) => state.treeNonce);

  return (
    <div className="h-full flex flex-col">
      <div className="py-2 h-14 flex items-center justify-center space-x-2">
        <ObjectsButton />
      </div>

      <div className="pt-2 pr-4 w-full h-full overflow-y-auto">
        <TreeMenuItem />
      </div>
    </div>
  );
}
